use std::fs;

use log::info;

use crate::{
    directory::{
        get_autopilot_path, get_config_path, get_jobs_path, get_logs_path, get_state_path,
    },
    job::get::get_jobs_paths,
    state::set::set_state_initial,
};

pub fn set_all_paths() {
    set_autopilot_path();
    set_logs_path();
    set_config_path();
    set_jobs_path();
    set_state_path();
}

pub fn set_autopilot_path() {
    fs::create_dir_all(&get_autopilot_path(None)).expect("Failed to create auto_pilot directory");
}

pub fn set_logs_path() {
    let logs_path: String = get_logs_path();
    fs::create_dir_all(&logs_path).expect("Failed to create logs directory");
}

pub fn set_config_path() {
    let conf_path: String = get_config_path();
    if fs::exists(&conf_path).unwrap() {
        info!("Config already exist")
    } else {
        fs::write(&conf_path, "{}").expect("Failed to create configuration file");
    }
}

pub fn set_state_path() {
    let state_path: String = get_state_path();
    if fs::exists(&state_path).unwrap() {

        // fs::remove_file(&state_path).expect("Failed to remove state file");
        // fs::write(&state_path, "{}").expect("Failed to create state file");
    } else {
        fs::write(&state_path, "{}").expect("Failed to create state file");
    }
}

pub fn set_jobs_path() {
    let jobs_path: String = get_jobs_path();
    fs::create_dir_all(&jobs_path).expect("Failed to create jobs directory");
}
