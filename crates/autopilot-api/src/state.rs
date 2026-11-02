use serde::Serialize;
use std::sync::Arc;
use tokio::sync::RwLock;

use autopilot_core::autopilot::AutoPilot;

// Simple response types
#[derive(Serialize)]
pub struct StatusResponse {
    pub running: bool,
    pub job_count: usize,
}

#[derive(Serialize)]
pub struct ReloadResponse {
    pub success: bool,
    pub message: String,
}

pub type RestartResponse = ReloadResponse;

// Shared application state
#[derive(Clone)]
pub struct AppState {
    pub auto_pilot: Arc<RwLock<AutoPilot>>,
    /// Bearer token required for every route except `/health`. `start_api`
    /// replaces this with the persisted or environment-provided token.
    pub api_token: Arc<String>,
}

impl AppState {
    pub fn new(auto_pilot: Arc<RwLock<AutoPilot>>) -> Self {
        Self {
            auto_pilot,
            api_token: Arc::new(String::new()),
        }
    }
}

#[cfg(test)]
#[path = "state.spec.rs"]
mod spec;
