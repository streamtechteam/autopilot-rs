use tokio_cron_scheduler::JobScheduler;

pub async fn init_time_check() -> JobScheduler {
    let scheduler = JobScheduler::new().await.unwrap();

    scheduler.start().await.unwrap();
    scheduler
}
