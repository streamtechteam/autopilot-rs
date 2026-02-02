use crate::{
    autopilot::AutoPilot,
    directory::{get_autopilot_path, set_all_paths},
    job::get::get_jobs,
    language::{self, en_us::AUTOPILOT_SHUTDOWN},
    state::set::set_state_initial,
};
use colored::*;
use log::{error, info, warn};
use std::panic::{self, AssertUnwindSafe};
use tokio::{self, signal};
// use tokio_cron_scheduler::JobScheduler;

pub async fn serve(config_path: Option<String>) {
    let mut auto_pilot = AutoPilot::new().await;
    auto_pilot.start();
    let scheduler = &auto_pilot.scheduler;
    // get_autopilot_path(config_path);
    set_all_paths();

    // Get jobs from JSON files and run them
    let jobs = get_jobs();

    for job in jobs {
        let scheduler = scheduler.clone();
        tokio::task::spawn(async move {
            // let result = panic::catch_unwind(AssertUnwindSafe(|| {
            job.run(&scheduler).await; // could panic
            // }));

            // match result {
            //     Ok(_) => info!(
            //         "{} : {}",
            //         language::en_us::JOB_LOAD_SUCCESS.green().bold(),
            //         job.name.green().bold()
            //     ),
            //     Err(_) => error!(
            //         "{} : {}",
            //         language::en_us::JOB_LOAD_FAILED.red().bold(),
            //         job.name.red().bold()
            //     ),
            // }
        });
    }

    // Keep the daemon running until Ctrl+C is pressed
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    set_state_initial().expect("Failed to initialize state");
    warn!("{}", AUTOPILOT_SHUTDOWN);
}
