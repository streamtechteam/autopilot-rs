use crate::{
    autopilot::AutoPilot, cli::handle_cli, cron::init::init_time_check, directory::get_directory,
    job::get::get_jobs, language, logging::init_logging,
};
use colored::*;
use log::{error, info, warn};
use std::panic::{self, AssertUnwindSafe};
use tokio::{self, signal};
use tokio_cron_scheduler::JobScheduler;

pub async fn serve(config_path: Option<String>) {
    let mut auto_pilot = AutoPilot::new().await;
    auto_pilot.start();
    let scheduler = &auto_pilot.scheduler;
    get_directory(config_path);

    // Get jobs from JSON files and run them
    let mut jobs = vec![];

    jobs = get_jobs(scheduler);

    for job in jobs {
        let scheduler = scheduler.clone();
        tokio::task::spawn(async move {
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                job.run(&scheduler); // could panic
            }));

            match result {
                Ok(_) => info!(
                    "{} : {}",
                    language::en_us::JOB_RUN_SUCCESS.green().bold(),
                    job.name.green().bold()
                ),
                Err(_) => error!(
                    "{} : {}",
                    language::en_us::JOB_RUN_FAILED.red().bold(),
                    job.name.red().bold()
                ),
            }
        });
    }
    // Keep the daemon running until Ctrl+C is pressed
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");

    warn!("Shutting down daemon...")
}
