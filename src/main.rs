use crate::{
    cli::handle_cli,
};
use log::warn;
use tokio::{self, signal};

mod autopilot;
mod cli;
mod conditions;
mod cron;
mod directory;
mod job;
mod language;
mod logging;
mod tasks;
mod utilities;

#[tokio::main]
async fn main() {
    handle_cli().await;

    // Keep the daemon running until Ctrl+C is pressed
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

    warn!("Shutting down daemon...");
}
