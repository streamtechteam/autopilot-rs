use std::sync::OnceLock;
pub mod get;
pub use get::*;
pub mod set;
pub use set::*;

pub static CONFIG_PATH: OnceLock<String> = OnceLock::new();

#[cfg(test)]
pub mod test_support {
    use crate::fs::CONFIG_PATH;
    use std::sync::{Mutex, MutexGuard, Once};

    static INIT: Once = Once::new();
    static LOCK: Mutex<()> = Mutex::new(());

    /// Initialise a shared config directory once per test process.
    pub fn config_dir() -> &'static str {
        INIT.call_once(|| {
            let dir = std::env::temp_dir().join(format!(
                "autopilot-unit-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_nanos()
            ));
            std::fs::create_dir_all(dir.join("jobs")).unwrap();
            std::fs::create_dir_all(dir.join("logs")).unwrap();
            CONFIG_PATH.set(dir.to_string_lossy().into_owned()).unwrap();
        });
        CONFIG_PATH.get().unwrap()
    }

    /// Serialise tests that read or write the shared config directory.
    pub fn lock() -> MutexGuard<'static, ()> {
        config_dir();
        LOCK.lock().unwrap_or_else(|poisoned| poisoned.into_inner())
    }

    pub fn jobs_dir() -> std::path::PathBuf {
        std::path::PathBuf::from(config_dir()).join("jobs")
    }
}
