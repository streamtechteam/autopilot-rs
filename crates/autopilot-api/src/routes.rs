use autopilot_core::{
    error::AutoPilotError,
    network::{REMOTE_OPT_IN_ENV, address, host, is_loopback_host, remote_bind_allowed},
};
use autopilot_core_fs::auth::load_or_create_api_token;
use axum::{
    Router, middleware,
    routing::{delete, get, post, put},
};
use colored::Colorize;
use log::info;
use tokio::task::JoinHandle;

use crate::{handlers::*, state::AppState};

pub async fn start_api(mut state: AppState) -> Result<JoinHandle<()>, AutoPilotError> {
    let bind_host = host();
    if !is_loopback_host(&bind_host) && !remote_bind_allowed() {
        return Err(AutoPilotError::Api(format!(
            "Refusing to bind the API to non-loopback host '{bind_host}'. Set {REMOTE_OPT_IN_ENV}=1 only when you intentionally expose it behind a trusted, TLS-terminating proxy."
        )));
    }

    let token =
        load_or_create_api_token().map_err(|error| AutoPilotError::Api(error.to_string()))?;
    state.api_token = std::sync::Arc::new(token);

    let app = Router::new()
        .route("/shutdown", post(shutdown))
        .route("/health", get(health))
        .route("/status", get(jobs_status))
        .route("/start", post(jobs_start))
        .route("/stop", post(jobs_stop))
        .route("/reload", post(jobs_reload))
        .route("/restart", post(jobs_restart))
        .route("/jobs", get(jobs_list))
        .route("/jobs", post(jobs_create))
        .route("/jobs/{id}", get(jobs_getbyid))
        .route("/jobs/{id}", delete(jobs_delete))
        .route("/jobs/{id}", put(jobs_update))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            crate::auth::require_auth,
        ))
        .with_state(state.clone());
    let address = address();
    let listener = tokio::net::TcpListener::bind(&address)
        .await
        .map_err(|err| AutoPilotError::Api(err.to_string()))?;

    info!("{}{}", "Api server started on ".green(), address);
    // Spawn API server
    let api_handle = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("API server failed");
    });
    Ok(api_handle)
}

#[cfg(test)]
#[path = "routes.spec.rs"]
mod spec;
