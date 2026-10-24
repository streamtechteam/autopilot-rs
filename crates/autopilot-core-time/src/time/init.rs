use tokio_cron_scheduler::JobScheduler;

use autopilot_core_error::AutoPilotError;

pub async fn init_time_check() -> Result<JobScheduler, AutoPilotError> {
    let scheduler = JobScheduler::new().await.map_err(AutoPilotError::Cron)?;

    scheduler.start().await.map_err(AutoPilotError::Cron)?;
    Ok(scheduler)
}

#[cfg(test)]
#[path = "init.spec.rs"]
mod spec;
