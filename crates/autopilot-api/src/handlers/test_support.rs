use std::sync::{Mutex, MutexGuard, Once};

use autopilot_core::autopilot::AutoPilot;
use autopilot_core::job::{Job, JobScheme};

use crate::state::AppState;

static CONFIG_INIT: Once = Once::new();
static CONFIG_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn config_lock() -> MutexGuard<'static, ()> {
    CONFIG_INIT.call_once(|| {
        let dir = std::env::temp_dir().join(format!(
            "autopilot-api-spec-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        autopilot_core::fs::init(Some(dir)).unwrap();
    });

    CONFIG_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub(crate) fn job(id: &str, name: &str) -> Job {
    Job::new(
        id.to_string(),
        name.to_string(),
        "Description".to_string(),
        None,
        None,
        vec![],
        vec![],
    )
}

pub(crate) fn payload(id: &str, name: &str) -> JobScheme {
    JobScheme {
        id: id.to_string(),
        name: Some(name.to_string()),
        description: Some("description".to_string()),
        when: None,
        check_interval: None,
        conditions: vec![],
        tasks: vec![],
    }
}

pub(crate) async fn state_with_jobs(jobs: Vec<Job>) -> AppState {
    let mut autopilot = AutoPilot::new().await.unwrap();
    autopilot.jobs = jobs;
    AppState {
        auto_pilot: std::sync::Arc::new(tokio::sync::RwLock::new(autopilot)),
    }
}
