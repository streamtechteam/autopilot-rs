use colored::*;
use std::fs;

// use serde_json::value;

use crate::{
    directory::get_state_path,
    state::{JobStatus, Status, StatusLog, set::set_state_initial},
    utilities,
};

pub fn get_status_log() -> StatusLog {
    let state_path = get_state_path();
    let state_string = fs::read_to_string(state_path).expect("Failed to read state file");
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
                set_state_initial().expect("failed to set state.json to inital state");
                // vec![]
                get_status_log()
            }
        };
    status_log
}

pub fn get_job_status(id: String) -> Status {
    let status_log = get_status_log();
    // println!("{:?}", status_log);
    status_log
        .statuses
        .iter()
        .find(|job| job.id == id)
        .map_or(Status::Unknown, |job| job.status.clone())
}
