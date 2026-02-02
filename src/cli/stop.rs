use colored::Colorize;
use duct::cmd;
use log::info;

use crate::{language, state::set::set_state_initial};

pub fn stop() {
    info!("{}", language::en_us::AUTOPILOT_SHUTDOWN.yellow());
    cmd("sh", vec!["-c", "kill $(pgrep auto_pilot)"])
        .read()
        .expect("failed to stop auto_pilot");
    set_state_initial().expect("Failed to initialize state");
}
