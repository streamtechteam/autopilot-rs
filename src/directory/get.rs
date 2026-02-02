use std::{env, fs};

use log::info;

pub fn get_autopilot_path(config_path: Option<String>) -> String {
    let home_path = env::home_dir().expect("Couldnt get home dir");
    let auto_pilot_path: String = home_path.to_str().unwrap().to_string()
        + config_path
            .unwrap_or("/.config/auto-pilot/".to_string())
            .as_str();

    auto_pilot_path
}

pub fn get_logs_path() -> String {
    get_autopilot_path(None) + "/logs"
}

pub fn get_config_path() -> String {
    get_autopilot_path(None) + "/autopilot.jsonc"
}

pub fn get_status_path() -> String {
    get_autopilot_path(None) + "/status.jsonc"
}

pub fn get_jobs_path() -> String {
    get_autopilot_path(None) + "/jobs"
}
