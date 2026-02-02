use crate::{cli::handle_cli, directory::set_all_paths, status::set::set_status_initial};
use tokio::{self, signal};

mod autopilot;
mod cli;
mod conditions;
mod cron;
mod directory;
mod job;
mod language;
mod logging;
mod status;
mod tasks;
mod utilities;

#[tokio::main]
async fn main() {
    set_all_paths(false);
    handle_cli().await;
}
