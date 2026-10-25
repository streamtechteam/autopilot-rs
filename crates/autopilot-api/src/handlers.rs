use std::{collections::HashMap, time::Duration};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use log::{info, warn};
use serde::Serialize;

use autopilot_core::{
    error::AutoPilotError,
    job::{
        JobScheme,
        get::get_jobs,
        set::{add_job, remove_job, update_job},
    },
    status::JobStatusEnum,
};

#[cfg(test)]
use autopilot_core::status::StateManager;

pub use autopilot_core::checks::{API_SERVICE_NAME, HealthResponse};

use crate::state::{AppState, ReloadResponse, RestartResponse, StatusResponse};

#[derive(Serialize, Debug)]
pub struct JobResponse {
    id: String,
    name: String,
    description: String,
    loaded: bool,
    status: String,
}

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: API_SERVICE_NAME.to_string(),
    })
}

pub async fn jobs_status(State(state): State<AppState>) -> Json<StatusResponse> {
    let ap = state.auto_pilot.read().await;
    Json(StatusResponse {
        running: ap.started,
        job_count: ap.jobs.len(),
    })
}

pub async fn jobs_start(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut ap = state.auto_pilot.write().await;
    if ap.started {
        return Err(StatusCode::CONFLICT);
    }

    if let Err(error) = ap.load_jobs().await {
        warn!("Failed to load jobs: {error}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    if let Err(error) = ap.init_status().await {
        warn!("Failed to initialize job status: {error}");
        return Err(StatusCode::INTERNAL_SERVER_ERROR);
    }
    ap.start();

    info!("Jobs started via API");
    Ok(Json(
        serde_json::json!({ "success": true, "message": "Jobs started" }),
    ))
}

pub async fn jobs_stop(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut ap = state.auto_pilot.write().await;
    if !ap.started {
        return Err(StatusCode::CONFLICT);
    }

    ap.stop_jobs()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    info!("Jobs stopped via API");
    Ok(Json(
        serde_json::json!({ "success": true, "message": "Jobs stopped" }),
    ))
}

pub async fn shutdown(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut ap = state.auto_pilot.write().await;
    ap.shutdown()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    drop(ap);

    warn!("Autopilot shut down via API");
    tokio::spawn(async {
        tokio::time::sleep(Duration::from_millis(100)).await;
        std::process::exit(0);
    });
    Ok(Json(
        serde_json::json!({ "success": true, "message": "Autopilot shutting down" }),
    ))
}

pub async fn jobs_reload(
    State(state): State<AppState>,
) -> Result<Json<ReloadResponse>, StatusCode> {
    let mut ap = state.auto_pilot.write().await;

    if let Err(e) = ap.reload().await {
        warn!("Reload failed: {}", e);
        return Ok(Json(ReloadResponse {
            message: format!("Reload failed: {}", e),
            success: false,
        }));
    }

    Ok(Json(ReloadResponse {
        message: "Config reloaded.".to_string(),
        success: true,
    }))
}

pub async fn jobs_restart(
    State(state): State<AppState>,
) -> Result<Json<RestartResponse>, StatusCode> {
    let mut ap = state.auto_pilot.write().await;

    if let Err(e) = ap.restart().await {
        warn!("Restart failed: {}", e);
        return Ok(Json(RestartResponse {
            message: format!("Restart failed: {}", e),
            success: false,
        }));
    }

    Ok(Json(RestartResponse {
        message: "Autopilot restarted.".to_string(),
        success: true,
    }))
}

// ============ Job CRUD Handlers ============

impl JobResponse {
    #[cfg(test)]
    async fn from_job_and_sm(job: &autopilot_core::job::Job, sm: StateManager) -> Self {
        JobResponse {
            id: job.id.clone(),
            loaded: job.loaded,
            name: job.name.clone(),
            description: job.description.clone(),
            status: format!("{:?}", sm.get_state_by_id(job.id.clone()).await),
        }
    }
}

impl From<&autopilot_core::status::JobStatusStruct> for JobResponse {
    fn from(job: &autopilot_core::status::JobStatusStruct) -> Self {
        JobResponse {
            id: job.id.clone(),
            name: job.name.clone(),
            loaded: true,
            description: job.name.clone(),
            status: format!("{:?}", job.status),
        }
    }
}

pub async fn jobs_list(
    State(state): State<AppState>,
) -> Result<Json<Vec<JobResponse>>, StatusCode> {
    let jobs = tokio::task::spawn_blocking(|| get_jobs(true))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|error| {
            warn!("Failed to load jobs: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let ap = state.auto_pilot.read().await;
    let statuses = ap.status_manager.get_status_log().await.statuses;
    let status_by_id: HashMap<&str, &JobStatusEnum> = statuses
        .iter()
        .map(|status| (status.id.as_str(), &status.status))
        .collect();

    let result = jobs
        .into_iter()
        .map(|job| {
            let status = status_by_id
                .get(job.id.as_str())
                .map(|status| format!("{status:?}"))
                .unwrap_or_else(|| "None".to_string());
            let loaded = status != "None";
            JobResponse {
                id: job.id,
                name: job.name,
                description: job.description,
                loaded,
                status,
            }
        })
        .collect();

    Ok(Json(result))
}

/// POST /jobs - Create a new job
pub async fn jobs_create(
    Json(payload): Json<JobScheme>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let JobScheme {
        name,
        description,
        when,
        check_interval,
        conditions,
        tasks,
        ..
    } = payload;

    let result = tokio::task::spawn_blocking(move || {
        add_job(name, description, when, check_interval, conditions, tasks)
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match result {
        Ok(path) => {
            info!("Created job via API: {:?}", path);
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Job created",
                "path": path.to_string_lossy()
            })))
        }
        Err(e) => {
            info!("Failed to create job: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// GET /jobs/{id} - Get job by ID
pub async fn jobs_getbyid(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<JobResponse>, StatusCode> {
    let jobs = tokio::task::spawn_blocking(|| get_jobs(true))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .map_err(|error| {
            warn!("Failed to load jobs: {error}");
            StatusCode::INTERNAL_SERVER_ERROR
        })?;
    let job = jobs
        .into_iter()
        .find(|job| job.id == id)
        .ok_or(StatusCode::NOT_FOUND)?;

    let ap = state.auto_pilot.read().await;
    let loaded = ap
        .jobs
        .iter()
        .find(|job| job.id == id)
        .is_some_and(|job| job.loaded);
    let status = ap
        .status_manager
        .get_state_by_id(id)
        .await
        .map(|status| format!("{status:?}"))
        .unwrap_or_else(|| "None".to_string());

    Ok(Json(JobResponse {
        id: job.id,
        name: job.name,
        description: job.description,
        loaded,
        status,
    }))
}

/// DELETE /jobs/{id} - Delete job by ID
pub async fn jobs_delete(Path(id): Path<String>) -> Result<Json<serde_json::Value>, StatusCode> {
    let result = tokio::task::spawn_blocking(move || remove_job(Some(id), None))
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match result {
        Ok(()) => {
            info!("Deleted job via API");
            Ok(Json(
                serde_json::json!({ "success": true, "message": "Job deleted" }),
            ))
        }
        Err(AutoPilotError::Job(message)) => {
            info!("Failed to delete job: {message}");
            Err(StatusCode::NOT_FOUND)
        }
        Err(AutoPilotError::InvalidJob(message)) => {
            info!("Invalid delete request: {message}");
            Err(StatusCode::BAD_REQUEST)
        }
        Err(error) => {
            warn!("Failed to delete job: {error}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// PUT /jobs/{id} - Update job by ID
pub async fn jobs_update(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<JobScheme>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let JobScheme {
        name,
        description,
        when,
        check_interval,
        conditions,
        tasks,
        ..
    } = payload;

    let result = tokio::task::spawn_blocking(move || {
        update_job(
            &id,
            name,
            description,
            when,
            check_interval,
            conditions,
            tasks,
        )
    })
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match result {
        Ok(path) => {
            info!("Updated job via API: {:?}", path);
            Ok(Json(serde_json::json!({
                "success": true,
                "message": "Job updated",
                "path": path.to_string_lossy()
            })))
        }
        Err(AutoPilotError::Job(message)) => {
            info!("Failed to update job: {message}");
            Err(StatusCode::NOT_FOUND)
        }
        Err(AutoPilotError::InvalidJob(message)) => {
            info!("Invalid update request: {message}");
            Err(StatusCode::BAD_REQUEST)
        }
        Err(error) => {
            warn!("Failed to update job: {error}");
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

#[cfg(test)]
#[path = "handlers.spec.rs"]
mod spec;
