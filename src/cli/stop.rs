use colored::Colorize;
use duct::cmd;
use log::info;

use crate::{language, status::set::set_status_initial};

pub fn stop(quiet: bool) {
    if !quiet {
        info!("{}", language::en_us::AUTOPILOT_SHUTDOWN.yellow());
    }
    cmd("sh", vec!["-c", "kill $(pgrep autopilot)"])
        .read()
        .expect("failed to stop auto_pilot");
    set_status_initial().expect("Failed to initialize state");
}
