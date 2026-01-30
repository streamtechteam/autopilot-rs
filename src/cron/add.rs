use std::error::Error;
use tokio_cron_scheduler::JobScheduler;

use crate::job::Job as JobStruct;

pub async fn add_job(
    job: &JobStruct,
    scheduler: &JobScheduler,
    run_job: impl Fn(&JobStruct) + Send + Sync + Clone + 'static,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Validate job time
    let target_time = job.when.ok_or_else(|| "Job time is None".to_string())?;

    let now = chrono::Local::now();
    if target_time <= now {
        return Err("Job time is in the past".into());
    }

    // Compute duration until the job should run
    let duration = (target_time - now)
        .to_std()
        .map_err(|e| format!("Failed to compute duration: {}", e))?;

    println!("Instant: {:?}", now);
    println!("Duration: {:?}", duration);
    // Duration::into();

    // Clone the job to pass to the async closure which requires 'static lifetime
    let job_clone = job.clone();
    let run_job_clone = run_job.clone();

    // Create one-shot job

    let cron_job = tokio_cron_scheduler::Job::new_one_shot_async(duration, move |_, _| {
        println!("Time {:?}", chrono::Local::now());
        let job = job_clone.clone();
        let run_job = run_job_clone.clone();

        Box::pin(async move {
            run_job(&job);
        })
    })?;
    // Add job to scheduler
    scheduler.add(cron_job).await?;

    Ok(())
}
