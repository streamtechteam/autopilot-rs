use std::{env, fs};

use log::info;

pub fn get_directory(config_path: Option<String>) -> String {
    let home_path = env::home_dir().expect("Couldnt get home dir");
    let auto_pilot_path: String = home_path.to_str().unwrap().to_string()
        + config_path
            .unwrap_or("/.config/auto-pilot/".to_string())
            .as_str();
    let jobs_path: String = auto_pilot_path.clone() + "/jobs";
    let logs_path: String = auto_pilot_path.clone() + "/logs";
    let conf_path: String = auto_pilot_path.clone() + "/autopilot.jsonc";

    fs::create_dir_all(&auto_pilot_path).expect("Failed to create auto_pilot directory");
    fs::create_dir_all(&jobs_path).expect("Failed to create jobs directory");
    fs::create_dir_all(&logs_path).expect("Failed to create logs directory");

    if fs::exists(&conf_path).unwrap() {
        info!("Config already exist")
    } else {
        fs::write(&conf_path, "{}").expect("Failed to create configuration file");
    }
    auto_pilot_path
}

pub fn get_logs_directory() -> String {
    let logs_path = get_directory(None) + "/logs";

    logs_path
}

pub fn get_config_path() -> String {
    let config_path = get_directory(None) + "/auto_pilot.json";

    config_path
}

pub fn get_jobs_directory() -> String {
    let jobs_path = get_directory(None) + "/jobs";

    jobs_path
}
