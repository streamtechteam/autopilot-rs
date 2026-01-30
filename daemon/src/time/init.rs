use std::error::Error;

use futures::future::BoxFuture;
use tokio_cron_scheduler::{Job, JobScheduler};

use crate::job::Job as JobStruct;

pub async fn init_time_check() -> JobScheduler {
    let scheduler = JobScheduler::new().await.unwrap();

    // let job = Job::new_async("*/5 * * * * *", |_uuid, _l| {
    //     Box::pin(async move {

    //         println!("Job is running every 5 seconds!");
    //     })
    // }).unwrap();
    let job = Job::new("*/5 * * * * *", |_uuid, _l| {
        println!("Job is running every 5 seconds!");
    })
    .unwrap();

    // Add job to scheduler
    scheduler.add(job).await.unwrap();

    // Start the scheduler
    scheduler.start().await.unwrap();
    scheduler
}

// pub async fn add_job(
//     job: &JobStruct,
//     scheduler: &JobScheduler,
//     run_job: impl Fn(&JobStruct) + Send + Sync + 'static,
// ) -> Result<(), Box<dyn Error>> {
//     let target_time = match job.when {
//         Some(dt) => dt,
//         None => return Err("Job time is None".into()),
//     };

//     let now = chrono::Local::now();
//     if target_time <= now {
//         return Err("Job time is in the past".into());
//     }

//     // Compute duration until the job should run
//     let duration = (target_time - now).to_std()?;
//     let job = job.clone();
//     // Create one-shot job

//     let cron_job = tokio_cron_scheduler::Job::new_one_shot(duration, move |_, _| {
//         run_job(job);
//     })?;

//     // Add job to scheduler
//     scheduler.add(cron_job).await?;

//     Ok(())
// }
pub async fn add_job(
    job: &JobStruct,
    scheduler: &JobScheduler,
    run_job: impl Fn(&JobStruct) + Send + Sync + Clone + 'static,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    // Validate job time
    let target_time = job.when.ok_or("Job time is None")?;

    let now = chrono::Local::now();
    if target_time <= now {
        return Err("Job time is in the past".into());
    }

    // Compute duration until the job should run
    let duration = (target_time - now)
        .to_std()
        .map_err(|e| format!("Failed to compute duration: {}", e))?;

    let job_clone = job.clone();
    let run_job_clone = run_job.clone();

    // Create one-shot job
    let cron_job = tokio_cron_scheduler::Job::new_one_shot_async(duration, move |_, _| {
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
// pub fn add_job(job: JobStruct,scheduler: &JobScheduler, run_job: impl Fn(&JobStruct) + Send + Sync + 'static)  -> Result<(), Box<dyn Error>> {
//     let target_time = &job.when.clone().unwrap();

//     // let dt: DateTime<Local> = Local::now() + chrono::Duration::seconds(30);
//       let now = chrono::Local::now();
//       let duration = (cron - now).to_std().unwrap();

//     let cron_job = Job::new_one_shot(duration, |_, _| {
//         run_job(&job);
//     }).unwrap();
//     scheduler.add(cron_job);
//     Ok(())
// }
