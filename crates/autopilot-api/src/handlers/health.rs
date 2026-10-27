use axum::Json;

pub use autopilot_core::checks::{API_SERVICE_NAME, HealthResponse};

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: API_SERVICE_NAME.to_string(),
    })
}

#[cfg(test)]
#[path = "health.spec.rs"]
mod spec;
