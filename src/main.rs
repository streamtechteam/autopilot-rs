use crate::{cli::handle_cli, directory::set_all_paths};
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
mod state;
mod tasks;
mod utilities;

#[tokio::main]
async fn main() {
    set_all_paths();

    handle_cli().await;
}
