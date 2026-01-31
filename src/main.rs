use crate::{
    cli::handle_cli, cron::init::init_time_check, directory::get_directory, job::get::get_jobs,
    logging::init_logging,
};
use colored::*;
use log::{error, info, warn};
use std::panic::{self, AssertUnwindSafe};
use tokio::{self, signal};
use tokio_cron_scheduler::JobScheduler;

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
}
