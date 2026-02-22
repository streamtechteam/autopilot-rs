use axum::{
    Router,
    routing::{get, post},
};
use log::info;
use ratatui::style::Stylize;
use tokio::task::JoinHandle;

use crate::api::handlers::*;
use crate::api::state::AppState;

pub async fn start_api(state: AppState) -> JoinHandle<()> {
    let app = Router::new()
        .route("/health", get(health))
        .route("/status", get(jobs_status))
        .route("/start", post(jobs_start))
        .route("/stop", post(jobs_stop))
        .route("/reload", post(jobs_reload))
        .route("/jobs", get(jobs_list))
        .route("/jobs", post(jobs_create))
        .route("/jobs/{id}", get(jobs_getbyid))
        .with_state(state.clone());

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .expect("Failed to bind API");
    info!("{}", "Api server started on 0.0.0.0:3000".green());
    // Spawn API server
    let api_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("API server failed");
    });
    api_handle
}
