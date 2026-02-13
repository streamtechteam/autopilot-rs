use tokio_cron_scheduler::JobScheduler;

use crate::error::AutoPilotError;

pub async fn init_time_check() -> Result<JobScheduler, AutoPilotError> {
    let scheduler = JobScheduler::new()
        .await
        .map_err(|err| AutoPilotError::Cron(err))?;

    scheduler
        .start()
        .await
        .map_err(|err| AutoPilotError::Cron(err))?;
    Ok(scheduler)
}
