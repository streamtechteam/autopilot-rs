use crate::{
    autopilot::AutoPilot,
    directory::{get_autopilot_path, set_all_paths},
    job::get::get_jobs,
    language::{self, en_us::AUTOPILOT_SHUTDOWN},
    status::set::set_status_initial,
};
use colored::*;
use log::{error, info, warn};
use std::panic::{self, AssertUnwindSafe};
use tokio::{self, signal};
// use tokio_cron_scheduler::JobScheduler;

pub async fn serve(config_path: Option<String>) {
    // let mut restart = true;
    loop {
        let mut auto_pilot = AutoPilot::new().await;
        auto_pilot.start();
        let scheduler = &auto_pilot.scheduler;
        // get_autopilot_path(config_path);
        set_all_paths(false, config_path.clone());

        // Get jobs from JSON files and run them
        let jobs = get_jobs(false);

        for job in jobs {
            let scheduler = scheduler.clone();
            tokio::task::spawn(async move {
                job.run(&scheduler, false).await;
            });
        }

        // Keep the daemon running until Ctrl+C is pressed
        // signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
        // set_state_initial().expect("Failed to initialize state");
        //
        // Handle SIGTERM signal
        #[cfg(unix)]
        {
            use tokio::signal::unix::{SignalKind, signal};

            // Handle SIGTERM
            let mut sigterm =
                signal(SignalKind::terminate()).expect("Failed to create SIGTERM listener");

            let sigterm_handle = tokio::spawn(async move {
                sigterm.recv().await;
                println!("Received SIGTERM, resetting status...");
                set_status_initial().expect("Failed to initialize status");
                std::process::exit(0);
            });

            let mut sighup =
                signal(SignalKind::hangup()).expect("Failed to create SIGHUP listener");
            let sighup_handle = tokio::spawn(async move {
                sighup.recv().await;
                println!("Received SIGHUP, resetting status...");
                set_status_initial().expect("Failed to initialize status");

                // std::process::exit(0);
                // serve().await;
            });

            // Handle SIGINT (Ctrl+C)
            let sigint_handle = tokio::spawn(async {
                signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
                println!("Received SIGINT, resetting status...");
                set_status_initial().expect("Failed to initialize status");
                std::process::exit(0);
            });
            // Wait for either signal handler to potentially terminate the process
            // In normal operation, handle_cli() would run indefinitely if it's a service
            tokio::try_join!(sigterm_handle, sigint_handle, sighup_handle).ok();
            // tokio::select! {
            //     _ = sigint_handle =>{

            //     }
            //     _ = sighup_handle =>{

            //     }
            //     _ = sigterm_handle =>{

            //     }
            // }
        }

        #[cfg(windows)]
        {
            // Handle Windows (only Ctrl+C)
            // tokio::spawn(async {
            signal::ctrl_c().await.expect("Failed to listen for ctrl+c");
            // println!("Received SIGINT/Ctrl+C, initializing state...");
            warn!("{}", AUTOPILOT_SHUTDOWN);
            set_status_initial().expect("Failed to initialize state");
            std::process::exit(0);
            // });
        }
    }
}
