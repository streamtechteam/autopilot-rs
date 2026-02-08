use crate::{cli::handle_cli, fs::set_all_paths};
use tokio::{self};

mod autopilot;
mod cli;
mod conditions;
mod cron;
mod cross_platform;
mod error;
mod fs;
mod job;
mod language;
mod logging;
mod status;
mod task;
mod utilities;

#[tokio::main]
async fn main() {
    if let Err(e) = set_all_paths(false, None) {
        eprintln!("Failed to set up directories: {}", e);
        std::process::exit(1);
    }
    handle_cli().await;
}
