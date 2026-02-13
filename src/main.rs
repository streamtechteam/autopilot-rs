use crate::cli::handle_cli;
use tokio::{self};

mod autopilot;
mod cli;
mod conditions;
mod cross_platform;
mod error;
mod fs;
mod job;
mod language;
mod logging;
mod status;
mod task;
mod time;
mod utilities;

#[tokio::main]
async fn main() {
    handle_cli().await;
}
