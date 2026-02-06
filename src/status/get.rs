use chrono::Local;
use log::error;
use std::fs;

// use serde_json::value;

use crate::{
    fs::get_status_path,
    status::{JobStatusEnum, StatusLog, set::set_status_initial},
    utilities,
};

pub fn get_status_log() -> StatusLog {
    let state_path = get_status_path();
    let state_string = match fs::read_to_string(state_path) {
        Ok(content) => content,
        Err(e) => {
            if log::log_enabled!(log::Level::Error) {
                error!("Failed to read state file: {}", e);
            } else {
                eprintln!("Failed to read state file: {}", e);
            }
            // Initialize status and retry
            if let Err(init_e) = set_status_initial() {
                if log::log_enabled!(log::Level::Error) {
                    error!("Failed to initialize state: {}", init_e);
                } else {
                    eprintln!("Failed to initialize state: {}", init_e);
                }
                // Return a default empty status log
                return StatusLog {
                    time: Local::now().to_rfc3339(),
                    statuses: vec![],
                };
            }
            return get_status_log();
        }
    };
    // println!("state {}", state_string);
    let status_log: StatusLog =
        match serde_json::from_str(utilities::jsonc_parser::jsonc_parse(&state_string).as_str()) {
            Ok(value) => value,
            Err(e) => {
                // Use the index `i` directly instead of searching again
                // println!(
                //     "Failed to parse state file: \n Error: {}",
                //     e.to_string().red()
                // );
                if let Err(init_e) = set_status_initial() {
                    if log::log_enabled!(log::Level::Error) {
                        error!("Failed to initialize state: {}", init_e);
                    } else {
                        eprintln!("Failed to initialize state: {}", init_e);
                    }
                    // Return a default empty status log
                    return StatusLog {
                        time: Local::now().to_rfc3339(),
                        statuses: vec![],
                    };
                }
                // vec![]
                get_status_log()
            }
        };
    status_log
}

pub fn get_job_status(id: String) -> JobStatusEnum {
    let status_log = get_status_log();
    // println!("{:?}", status_log);
    status_log
        .statuses
        .iter()
        .find(|job| job.id == id)
        .map_or(JobStatusEnum::Unknown, |job| job.status.clone())
}
