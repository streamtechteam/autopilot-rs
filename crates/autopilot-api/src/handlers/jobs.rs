use std::collections::HashMap;

use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};
use log::{info, warn};
use serde::Deserialize;

use autopilot_core::{
    conditions::ConditionScheme,
    error::AutoPilotError,
    job::{
        JobScheme, TaskMode,
        get::get_jobs,
        set::{add_job, remove_job, update_job},
    },
    status::JobStatusEnum,
    task::TaskScheme,
    time::When,
};

use super::types::JobResponse;
use crate::state::AppState;

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
        task_mode,
        ..
    } = payload;

    let result = tokio::task::spawn_blocking(move || {
        add_job(
            name,
            description,
            when,
            check_interval,
            conditions,
            tasks,
            task_mode,
        )
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

/// PUT body. `task_mode` is optional so an update can preserve the existing mode.
#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobUpdateRequest {
    #[serde(rename = "id")]
    _id: String,
    name: Option<String>,
    description: Option<String>,
    when: Option<When>,
    check_interval: Option<String>,
    conditions: Vec<ConditionScheme>,
    tasks: Vec<TaskScheme>,
    #[serde(default)]
    task_mode: Option<TaskMode>,
}

/// PUT /jobs/{id} - Update job by ID
pub async fn jobs_update(
    State(_state): State<AppState>,
    Path(id): Path<String>,
    Json(payload): Json<JobUpdateRequest>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let JobUpdateRequest {
        _id: _,
        name,
        description,
        when,
        check_interval,
        conditions,
        tasks,
        task_mode,
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
            task_mode,
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
#[path = "jobs.spec.rs"]
mod spec;
