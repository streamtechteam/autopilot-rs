use std::time::Duration;

use axum::{Json, extract::State, http::StatusCode};
use log::{info, warn};

use crate::state::{AppState, ReloadResponse, RestartResponse, StatusResponse};

pub async fn jobs_status(State(state): State<AppState>) -> Json<StatusResponse> {
    let ap = state.auto_pilot.read().await;
    Json(StatusResponse {
        running: ap.is_started(),
        job_count: ap.jobs.len(),
    })
}

pub async fn jobs_start(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut ap = state.auto_pilot.write().await;
    if ap.is_started() {
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
    ap.start_all();

    info!("Jobs started via API");
    Ok(Json(
        serde_json::json!({ "success": true, "message": "Jobs started" }),
    ))
}

pub async fn jobs_stop(
    State(state): State<AppState>,
) -> Result<Json<serde_json::Value>, StatusCode> {
    let mut ap = state.auto_pilot.write().await;
    if !ap.is_started() {
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

#[cfg(test)]
#[path = "lifecycle.spec.rs"]
mod spec;
