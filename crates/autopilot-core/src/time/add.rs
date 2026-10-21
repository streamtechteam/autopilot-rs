use std::future::Future;

use tokio_cron_scheduler::JobScheduler;

use crate::{
    error::AutoPilotError,
    job::Job,
    status::StateManager,
    time::{When, to_cron_expression},
};

pub async fn add_job<Fut, F>(
    job: &Job,
    scheduler: &JobScheduler,
    run_job: F,
    state_manager: StateManager,
) -> Result<(), AutoPilotError>
where
    Fut: Future<Output = ()> + Send + 'static,
    F: Fn(Job, StateManager) -> Fut + Send + Sync + Clone + 'static,
{
    // Validate job time
    let when = match job.when.clone() {
        Some(when) => when,
        None => {
            return Err(AutoPilotError::Time("Job time is None".to_string()));
        }
    };

    let cron_job: tokio_cron_scheduler::Job;

    match &when {
        When::Once(value) => {
            let target_time = value
                .parse()
                .map_err(|err| AutoPilotError::Time(format!("Failed to parse time : {}", err)))?;
            let now = chrono::Local::now();
            if target_time <= now {
                return Err(AutoPilotError::Time("Job time is in the past".to_string()));
            }

            let duration = (target_time - now)
                .to_std()
                .map_err(|e| AutoPilotError::Time(format!("Failed to compute duration: {}", e)))?;

            let job_clone = job.clone();
            let run_job_clone = run_job.clone();
            let sm_clone = state_manager.clone();
            cron_job = tokio_cron_scheduler::Job::new_one_shot_async(duration, move |_, _| {
                let job = job_clone.clone();
                let run_job = run_job_clone.clone();
                let sm = sm_clone.clone();
                Box::pin(async move {
                    run_job(job, sm).await;
                })
            })?;
            // Add job to scheduler
            scheduler.add(cron_job).await?;
        }
        When::Cron(expression) => {
            let job_clone = job.clone();
            let run_job_clone = run_job.clone();

            cron_job = tokio_cron_scheduler::Job::new_async(expression, move |_, _| {
                let job_clone = job_clone.clone();
                let run_job_clone = run_job_clone.clone();
                let state_manager_clone = state_manager.clone();
                Box::pin(async move {
                    run_job_clone(job_clone.clone(), state_manager_clone.clone()).await;
                })
            })?;
            scheduler.add(cron_job).await?;
        }
        _ => {
            let job_clone = job.clone();
            let run_job_clone = run_job.clone();
            let cron_expression = to_cron_expression(when.clone()).map_err(|e| {
                AutoPilotError::Time(format!("Failed to build cron expression: {}", e))
            })?;

            cron_job =
                tokio_cron_scheduler::Job::new_async(cron_expression.as_str(), move |_, _| {
                    let job_clone = job_clone.clone();
                    let run_job_clone = run_job_clone.clone();
                    let state_manager_clone = state_manager.clone();
                    Box::pin(async move {
                        run_job_clone(job_clone.clone(), state_manager_clone.clone()).await;
                    })
                })?;
            scheduler.add(cron_job).await?;
        }
    }

    Ok(())
}
