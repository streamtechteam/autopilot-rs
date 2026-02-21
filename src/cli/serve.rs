use crate::{
    autopilot::AutoPilot, status::set::set_status_initial,
};
use log::{error, info, warn};
use tokio::{self, signal};

pub async fn serve(verbose: bool) {
    let mut auto_pilot = AutoPilot::new().await;
    auto_pilot.start(verbose);

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
            warn!("{}", crate::language::en_us::AUTOPILOT_SHUTDOWN);
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
            auto_pilot.reload_config().await;
            // std::process::exit(0);
            // serve().await;
        });

        // Handle SIGINT (Ctrl+C)
        let sigint_handle = tokio::spawn(async {
            if let Err(e) = signal::ctrl_c().await {
                error!("Failed to listen for ctrl+c: {}", e);
                std::process::exit(1);
            }
            warn!("Received SIGINT, resetting status...");
            if let Err(e) = set_status_initial() {
                error!("Failed to initialize status: {}", e);
            }
            warn!("{}", crate::language::en_us::AUTOPILOT_SHUTDOWN);
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
