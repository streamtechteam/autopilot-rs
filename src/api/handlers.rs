use std::sync::atomic::Ordering;

use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::{get, post},
};
use log::info;
use serde::{Deserialize, Serialize};

use crate::api::state::AppState;

#[derive(Serialize)]
pub struct JobsStatus {
    running: bool,
    job_count: usize,
}

#[derive(Deserialize)]
pub struct StartConfig {
    verbose: Option<bool>,
}

pub async fn health() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "status": "ok", "service": "autopilot-api" }))
}

pub async fn jobs_status(State(state): State<AppState>) -> Json<JobsStatus> {
    let ap = state.auto_pilot.read().await;
    Json(JobsStatus {
        running: state.started.load(Ordering::Relaxed),
        job_count: ap.jobs.len(),
    })
}

pub async fn jobs_start(
    State(state): State<AppState>,
    Json(payload): Json<Option<StartConfig>>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    // Prevent double-start
    if state.started.swap(true, Ordering::Relaxed) {
        return Err(StatusCode::CONFLICT);
    }

    let mut ap = state.auto_pilot.write().await;
    ap.load_jobs();
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

pub async fn jobs_reload(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut ap = state.auto_pilot.write().await;
    ap.reload().await;
    // reload_config already calls run_jobs(), so ensure flag is set
    state.started.store(true, Ordering::Relaxed);

    Ok(Json(
        serde_json::json!({ "success": true, "message": "Config reloaded" }),
    ))
}
