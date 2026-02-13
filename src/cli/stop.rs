use colored::Colorize;
use log::info;

use crate::{language, status::set::set_status_initial};

pub fn stop(quiet: bool) {
    if !quiet {
        info!("{}", language::en_us::AUTOPILOT_SHUTDOWN.yellow());
    }
    if let Err(e) = duct_sh::sh_dangerous("kill $(pgrep autopilot)").read() {
        eprintln!("Warning: failed to stop auto_pilot: {}", e);
    }
    if let Err(e) = set_status_initial() {
        eprintln!("Warning: Failed to initialize state: {}", e);
    }
}
