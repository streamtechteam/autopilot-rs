use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard, Once},
};

use autopilot_core_fs::config;

static INIT: Once = Once::new();
static LOCK: Mutex<()> = Mutex::new(());

pub fn config_dir() -> &'static str {
    INIT.call_once(|| {
        let dir = std::env::temp_dir().join(format!(
            "autopilot-core-spec-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        config::initialize(Some(dir)).unwrap();
    });
    config::current().unwrap().root().to_str().unwrap()
}

pub fn lock() -> MutexGuard<'static, ()> {
    config_dir();
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn jobs_dir() -> PathBuf {
    config::current().unwrap().jobs_dir()
}
