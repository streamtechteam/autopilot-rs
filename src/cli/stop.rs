use colored::Colorize;
use duct::cmd;
use log::info;

use crate::language;

pub fn stop() {
    info!("{}", language::en_us::AUTOPILOT_SHUTDOWN.yellow());
    cmd("sh", vec!["-c", "kill $(pgrep auto_pilot)"]).read();
}
