use std::{
    fs,
    path::{Path, PathBuf},
    sync::OnceLock,
};

use crate::error::{FsError, Result};

static DIR_CONFIG: OnceLock<DirConfig> = OnceLock::new();

#[derive(Clone, Debug)]
pub struct DirConfig {
    root: PathBuf,
}

impl DirConfig {
    pub fn discover(custom: Option<PathBuf>) -> Result<Self> {
        let root = custom.unwrap_or_else(default_root);
        let paths = Self::from_root(root);
        paths.ensure()?;
        Ok(paths)
    }

    pub fn from_root(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn config_file(&self) -> PathBuf {
        self.root.join("autopilot.jsonc")
    }

    pub fn jobs_dir(&self) -> PathBuf {
        self.root.join("jobs")
    }

    pub fn logs_dir(&self) -> PathBuf {
        self.root.join("logs")
    }

    pub fn ensure(&self) -> Result<()> {
        fs::create_dir_all(&self.root).map_err(|error| {
            FsError::Init(format!(
                "Failed to create AutoPilot directory {}: {error}",
                self.root.display()
            ))
        })?;
        fs::create_dir_all(self.jobs_dir())
            .map_err(|error| FsError::Init(format!("Failed to create jobs directory: {error}")))?;
        fs::create_dir_all(self.logs_dir())
            .map_err(|error| FsError::Init(format!("Failed to create logs directory: {error}")))?;

        let config_file = self.config_file();
        if !config_file.exists() {
            fs::write(&config_file, "{}").map_err(|error| {
                FsError::Init(format!("Failed to create configuration file: {error}"))
            })?;
        }

        Ok(())
    }
}

pub fn initialize(custom: Option<PathBuf>) -> Result<&'static DirConfig> {
    if let Some(paths) = DIR_CONFIG.get() {
        paths.ensure()?;
        return Ok(paths);
    }

    let paths = DirConfig::discover(custom)?;
    let _ = DIR_CONFIG.set(paths);
    DIR_CONFIG.get().ok_or(FsError::NotInitialized)
}

pub fn current() -> Result<&'static DirConfig> {
    DIR_CONFIG.get().ok_or(FsError::NotInitialized)
}

fn default_root() -> PathBuf {
    let home = std::env::var_os("HOME")
        .or_else(|| std::env::var_os("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."));

    if cfg!(target_os = "windows") {
        home.join("AppData").join("Roaming").join("autopilot")
    } else {
        home.join(".config").join("autopilot")
    }
}

#[cfg(test)]
#[path = "config.spec.rs"]
mod spec;
