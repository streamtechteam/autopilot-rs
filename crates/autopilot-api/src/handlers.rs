use std::{collections::HashMap, sync::atomic::Ordering};

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use log::{info, warn};
use serde::Serialize;

use autopilot_core::{
    job::{
        JobScheme,
        get::get_jobs,
        set::{add_job, remove_job},
    },
    status::{JobStatusEnum, StateManager},
};

pub use autopilot_core::checks::{API_SERVICE_NAME, HealthResponse};

use crate::state::{AppState, ReloadResponse, StatusResponse};

#[derive(Serialize)]
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
        running: state.started.load(Ordering::Relaxed),
        job_count: ap.jobs.len(),
    })
}

pub async fn jobs_start(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Prevent double-start
    if state.started.swap(true, Ordering::Relaxed) {
        return Err(StatusCode::CONFLICT);
    }

    let mut ap = state.auto_pilot.write().await;
    ap.load_jobs()
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    ap.run_jobs();

    info!("Jobs started via API");
    Ok(Json(
        serde_json::json!({ "success": true, "message": "Jobs started" }),
    ))
}

pub async fn jobs_stop(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    if !state.started.swap(false, Ordering::Relaxed) {
        return Err(StatusCode::CONFLICT); // Already stopped
    }

    let mut ap = state.auto_pilot.write().await;
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
    if !state.started.swap(false, Ordering::Relaxed) {
        ap.shutdown()
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    }

    warn!("Autopilot shutted down via API");
    std::process::exit(0);
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

    // reload_config already calls run_jobs(), so ensure flag is set
    state.started.store(true, Ordering::Relaxed);
    Ok(Json(ReloadResponse {
        message: "Config reloaded.".to_string(),
        success: true,
    }))
}

// ============ Job CRUD Handlers ============

impl JobResponse {
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
            //TODO
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
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let ap = state.auto_pilot.read().await;
    let statuses = ap.status_manager.get_status_log().await.statuses;

    let live_map: HashMap<String, JobResponse> = statuses
        .iter()
        .map(|j| {
            let resp = JobResponse::from(j);
            (resp.id.clone(), resp)
        })
        .collect();

    let status_by_id: HashMap<&str, &JobStatusEnum> = statuses
        .iter()
        .map(|status| (status.id.as_str(), &status.status))
        .collect();

    let mut db_map = HashMap::new();
    for job in &jobs {
        let status = status_by_id
            .get(job.id.as_str())
            .map(|status| format!("Some({:?})", status))
            .unwrap_or_else(|| "None".to_string());
        let resp = JobResponse {
            id: job.id.clone(),
            name: job.name.clone(),
            description: job.description.clone(),
            loaded: job.loaded,
            status,
        };
        db_map.insert(resp.id.clone(), resp);
    }

    let mut merged = db_map;

    for (id, live_job) in live_map {
        merged.insert(id, live_job);
    }

    let result: Vec<JobResponse> = merged.into_values().collect();

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
    let ap = state.auto_pilot.read().await;
    if let Some(job) = ap.jobs.iter().find(|j| j.id == id) {
        let resp = JobResponse::from_job_and_sm(job, ap.status_manager.clone()).await;
        Ok(Json(resp))
    } else {
        Err(StatusCode::NOT_FOUND)
    }
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
        Err(e) => {
            info!("Failed to delete job: {}", e);
            Err(StatusCode::NOT_FOUND)
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
        description,
        when,
        check_interval,
        conditions,
        tasks,
        ..
    } = payload;

    let result = tokio::task::spawn_blocking(move || {
        if remove_job(Some(id.clone()), None).is_err() {
            return Err(StatusCode::NOT_FOUND);
        }

        add_job(
            Some(format!("{}_updated", id)),
            description,
            when,
            check_interval,
            conditions,
            tasks,
        )
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
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
        Err(e) => {
            info!("Failed to update job: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
