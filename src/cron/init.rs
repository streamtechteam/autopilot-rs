use tokio_cron_scheduler::JobScheduler;

pub async fn init_time_check() -> JobScheduler {
    let scheduler = JobScheduler::new().await.unwrap();

    // let job = Job::new_async("*/5 * * * * *", |_uuid, _l| {
    //     Box::pin(async move {

    //         println!("Job is running every 5 seconds!");
    //     })
    // }).unwrap();
    // let job = Job::new("*/5 * * * * *", |_uuid, _l| {
    //     println!("Job is running every 5 seconds!");
    // })
    // .unwrap();

    // Add job to scheduler
    // scheduler.add(job).await.unwrap();

    // Start the scheduler
    scheduler.start().await.unwrap();
    scheduler
}
