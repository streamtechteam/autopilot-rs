use super::*;

#[tokio::test]
async fn initialises_and_starts_a_scheduler() {
    let mut scheduler = init_time_check().await.unwrap();

    scheduler.shutdown().await.unwrap();
}

#[tokio::test]
async fn initialises_independent_schedulers() {
    let mut first = init_time_check().await.unwrap();
    let mut second = init_time_check().await.unwrap();

    first.shutdown().await.unwrap();
    second.shutdown().await.unwrap();
}
