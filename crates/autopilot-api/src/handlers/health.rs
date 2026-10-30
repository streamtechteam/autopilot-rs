use axum::Json;

pub use autopilot_core::health::HealthResponse;
pub use autopilot_core::network::API_SERVICE_NAME;

pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        service: API_SERVICE_NAME.to_string(),
    })
}

#[cfg(test)]
#[path = "health.spec.rs"]
mod spec;
