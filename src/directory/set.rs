use std::fs;

use log::info;

use crate::{
    directory::{
        get_autopilot_path, get_config_path, get_jobs_path, get_logs_path, get_status_path,
    },
    error::{AutoPilotError, Result},
};

pub fn set_all_paths(quiet: bool, config_path: Option<String>) -> Result<()> {
    set_autopilot_path(config_path)?;
    set_logs_path()?;
    set_config_path(quiet)?;
    set_jobs_path()?;
    set_status_path()?;
    Ok(())
}

pub fn set_autopilot_path(config_path: Option<String>) -> Result<()> {
    fs::create_dir_all(&get_autopilot_path(config_path))
        .map_err(|e| AutoPilotError::DirectoryInit(format!("Failed to create auto_pilot directory: {}", e)))
}

pub fn set_logs_path() -> Result<()> {
    let logs_path: String = get_logs_path();
    fs::create_dir_all(&logs_path)
        .map_err(|e| AutoPilotError::DirectoryInit(format!("Failed to create logs directory: {}", e)))
}

pub fn set_config_path(quiet: bool) -> Result<()> {
    let conf_path: String = get_config_path();
    let exists = fs::metadata(&conf_path).is_ok();
    if exists {
        if !quiet {
            info!("Config already exist")
        }
    } else {
        fs::write(&conf_path, "{}")
            .map_err(|e| AutoPilotError::DirectoryInit(format!("Failed to create configuration file: {}", e)))?;
    }
    Ok(())
}

pub fn set_status_path() -> Result<()> {
    let status: String = get_status_path();
    let exists = fs::metadata(&status).is_ok();
    if exists {
        // fs::remove_file(&state_path).expect("Failed to remove state file");
        // fs::write(&state_path, "{}").expect("Failed to create state file");
    } else {
        fs::write(&status, "{}")
            .map_err(|e| AutoPilotError::DirectoryInit(format!("Failed to create status file: {}", e)))?;
    }
    Ok(())
}

pub fn set_jobs_path() -> Result<()> {
    let jobs_path: String = get_jobs_path();
    fs::create_dir_all(&jobs_path)
        .map_err(|e| AutoPilotError::DirectoryInit(format!("Failed to create jobs directory: {}", e)))
}
