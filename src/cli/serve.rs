use crate::{
    autopilot::AutoPilot,
    cli::status::{check_if_running, status},
    fs::{CONFIG_PATH, set_all_paths},
    job::get::get_jobs,
    status::set::set_status_initial,
};
use colored::Colorize;
use log::{error, info, warn};
use tokio::{self, signal};
use tokio_cron_scheduler::job;

pub async fn serve(verbose: bool) {
    let mut auto_pilot = AutoPilot::new().await;
    auto_pilot.start(verbose);
    match check_if_running() {
        true => {
            error!("there is already an instance of Autopilot running");
            return;
        }
        _ => {}
    }

    let scheduler = &auto_pilot.scheduler;
    if let Err(e) = set_status_initial() {
        error!("Failed to initialize status: {}", e);
        return;
    }

    info!("{}", "Autopilot served!".green());
    // Get jobs from JSON files and run them
    let jobs = get_jobs(false);
    for job in jobs {
        let scheduler = scheduler.clone();
        tokio::task::spawn(async move {
            job.run(&scheduler, false).await;
        });
    }

    // Keep the daemon running until Ctrl+C is pressed
    // Handle SIGTERM signal
    #[cfg(unix)]
    {
        use tokio::signal::unix::{SignalKind, signal};

        // Handle SIGTERM
        let mut sigterm = match signal(SignalKind::terminate()) {
            Ok(sig) => sig,
            Err(e) => {
                error!("Failed to create SIGTERM listener: {}", e);
                std::process::exit(1);
            }
        };

        let sigterm_handle = tokio::spawn(async move {
            use log::info;

            sigterm.recv().await;
            info!("Received SIGTERM, resetting status...");
            if let Err(e) = set_status_initial() {
                error!("Failed to initialize status: {}", e);
            }
            std::process::exit(0);
        });

        let mut sighup = match signal(SignalKind::hangup()) {
            Ok(sig) => sig,
            Err(e) => {
                error!("Failed to create SIGHUP listener: {}", e);
                std::process::exit(1);
            }
        };

        let sighup_handle = tokio::spawn(async move {
            use log::info;

            sighup.recv().await;
            info!("Received SIGHUP, resetting status...");
            if let Err(e) = set_status_initial() {
                error!("Failed to initialize status: {}", e);
            }

            // std::process::exit(0);
            // serve().await;
        });

        // Handle SIGINT (Ctrl+C)
        let sigint_handle = tokio::spawn(async {
            if let Err(e) = signal::ctrl_c().await {
                error!("Failed to listen for ctrl+c: {}", e);
                std::process::exit(1);
            }
            println!("Received SIGINT, resetting status...");
            if let Err(e) = set_status_initial() {
                error!("Failed to initialize status: {}", e);
            }
            std::process::exit(0);
        });
        // Wait for either signal handler to potentially terminate the process
        // In normal operation, handle_cli() would run indefinitely if it's a service
        tokio::try_join!(sigterm_handle, sigint_handle, sighup_handle).ok();
    }

    #[cfg(windows)]
    {
        // Handle Windows (only Ctrl+C)
        if let Err(e) = signal::ctrl_c().await {
            error!("Failed to listen for ctrl+c: {}", e);
            std::process::exit(1);
        }
        warn!("{}", crate::language::en_us::AUTOPILOT_SHUTDOWN);
        if let Err(e) = set_status_initial() {
            error!("Failed to initialize status: {}", e);
        }
        std::process::exit(0);
    }
}
