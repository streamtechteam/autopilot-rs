use crate::{cli::handle_cli, directory::set_all_paths};
use tokio::{
    self,
};

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
    set_all_paths(false, None);
    handle_cli().await;
}
