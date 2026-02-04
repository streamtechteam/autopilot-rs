use std::env;


pub fn get_autopilot_path(config_path: Option<String>) -> String {
    let home_path = env::home_dir().expect("Couldnt get home dir");
    
    let default_subdir = if cfg!(target_os = "windows") {
        "/AppData/Roaming/auto-pilot"
    } else {
        "/.config/auto-pilot"
    };

    let auto_pilot_path: String = home_path.to_str().unwrap().to_string()
        + config_path
            .unwrap_or(default_subdir.to_string())
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
