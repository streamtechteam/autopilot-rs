use std::fs;

use log::info;

use crate::{
    directory::{
        get_autopilot_path, get_config_path, get_jobs_path, get_logs_path, get_status_path,
    },
    job::get::get_jobs_paths,
    status::set::set_status_initial,
};

pub fn set_all_paths(quiet: bool, config_path: Option<String>) {
    set_autopilot_path(config_path);
    set_logs_path();
    set_config_path(quiet);
    set_jobs_path();
    set_status_path();
}

pub fn set_autopilot_path(config_path: Option<String>) {
    fs::create_dir_all(&get_autopilot_path(config_path))
        .expect("Failed to create auto_pilot directory");
}

pub fn set_logs_path() {
    let logs_path: String = get_logs_path();
    fs::create_dir_all(&logs_path).expect("Failed to create logs directory");
}

pub fn set_config_path(quiet: bool) {
    let conf_path: String = get_config_path();
    if fs::exists(&conf_path).unwrap() {
        if !quiet {
            info!("Config already exist")
        }
    } else {
        fs::write(&conf_path, "{}").expect("Failed to create configuration file");
    }
}

pub fn set_status_path() {
    let status: String = get_status_path();
    if fs::exists(&status).unwrap() {

        // fs::remove_file(&state_path).expect("Failed to remove state file");
        // fs::write(&state_path, "{}").expect("Failed to create state file");
    } else {
        fs::write(&status, "{}").expect("Failed to create status file");
    }
}

pub fn set_jobs_path() {
    let jobs_path: String = get_jobs_path();
    fs::create_dir_all(&jobs_path).expect("Failed to create jobs directory");
}
