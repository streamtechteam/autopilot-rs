use std::{
    path::PathBuf,
    sync::{Mutex, MutexGuard, Once},
};

use autopilot_core_fs::{init, paths};

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
        init(Some(dir)).unwrap();
    });
    paths().unwrap().root().to_str().unwrap()
}

pub fn lock() -> MutexGuard<'static, ()> {
    config_dir();
    LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
}

pub fn jobs_dir() -> PathBuf {
    paths().unwrap().jobs_dir()
}
