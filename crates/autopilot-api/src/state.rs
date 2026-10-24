use serde::Serialize;
use std::sync::{Arc, atomic::AtomicBool};
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
    pub started: Arc<AtomicBool>,
}

#[cfg(test)]
#[path = "state.spec.rs"]
mod spec;
