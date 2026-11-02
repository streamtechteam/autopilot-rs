use std::sync::Arc;
use std::time::Duration;

use autopilot_api::{routes::start_api, state::AppState};
use autopilot_core::autopilot::AutoPilot;
use log::{error, info, warn};
use tokio::{self, signal, sync::RwLock};

async fn shutdown_and_exit(auto_pilot: Arc<RwLock<AutoPilot>>) {
    let result = tokio::time::timeout(Duration::from_secs(10), async {
        auto_pilot.write().await.shutdown().await
    })
    .await;

    match result {
        Ok(Ok(())) => {}
        Ok(Err(error)) => error!("Failed to stop jobs during shutdown: {error}"),
        Err(_) => error!("Timed out while stopping jobs during shutdown"),
    }

    warn!("{}", autopilot_core::language::en_us::AUTOPILOT_SHUTDOWN);
    std::process::exit(0);
}

pub async fn serve(verbose: bool, api: bool) {
    let auto_pilot = match AutoPilot::new().await {
        Ok(ap) => Arc::new(RwLock::new(ap)),
        Err(err) => {
            error!("Failed to initialize autopilot: {}", err);
            std::process::exit(1);
        }
    };
    if let Err(err) = auto_pilot.write().await.init(verbose).await {
        error!("Failed to initialize autopilot: {}", err);
        std::process::exit(1);
    }
    if !api {
        auto_pilot.write().await.start_all();
    }
    let state = AppState::new(auto_pilot.clone());
    match start_api(state).await {
        Ok(_) => {}
        Err(err) => {
            error!("Failed to start api : {}", err)
        }
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

        let sigterm_handle = {
            let auto_pilot = auto_pilot.clone();
            tokio::spawn(async move {
                sigterm.recv().await;
                info!("Received SIGTERM, stopping jobs...");
                shutdown_and_exit(auto_pilot).await;
            })
        };

        let mut sighup = match signal(SignalKind::hangup()) {
            Ok(sig) => sig,
            Err(e) => {
                error!("Failed to create SIGHUP listener: {}", e);
                std::process::exit(1);
            }
        };

        let sighup_handle = {
            let auto_pilot = auto_pilot.clone();
            tokio::spawn(async move {
                while sighup.recv().await.is_some() {
                    info!("Received SIGHUP, reloading...");
                    if let Err(e) = auto_pilot.write().await.reload().await {
                        error!("Failed to reload autopilot: {}", e);
                    }
                }
            })
        };

        // Handle SIGINT (Ctrl+C)
        let sigint_handle = {
            let auto_pilot = auto_pilot.clone();
            tokio::spawn(async move {
                if let Err(e) = signal::ctrl_c().await {
                    error!("Failed to listen for ctrl+c: {}", e);
                    std::process::exit(1);
                }
                info!("Received SIGINT, stopping jobs...");
                shutdown_and_exit(auto_pilot).await;
            })
        };
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
        info!("Received Ctrl+C, stopping jobs...");
        shutdown_and_exit(auto_pilot).await;
    }
}
