use super::*;
use crate::job::Job;
use autopilot_core_status::StateManager;

fn job(id: &str, when: Option<When>) -> Job {
    Job::new(
        id.to_string(),
        id.to_string(),
        String::new(),
        None,
        when,
        vec![],
        vec![],
    )
}

async fn run_noop(_job: Job, _state: StateManager) {}

#[tokio::test]
async fn rejects_jobs_without_a_time() {
    let mut scheduler = autopilot_core_time::init::init_time_check().await.unwrap();
    let error = add_job(
        &job("missing", None),
        &scheduler,
        run_noop,
        StateManager::new(),
    )
    .await
    .unwrap_err();

    assert!(error.to_string().contains("Job time is None"));
    scheduler.shutdown().await.unwrap();
}

#[tokio::test]
async fn rejects_past_one_shot_jobs() {
    let mut scheduler = autopilot_core_time::init::init_time_check().await.unwrap();
    let past = chrono::Local::now() - chrono::Duration::days(1);
    let when = When::Once(autopilot_core_time::DateTimeScheme {
        date: past.format("%Y/%m/%d").to_string(),
        time: past.format("%H:%M").to_string(),
    });

    let error = add_job(
        &job("past", Some(when)),
        &scheduler,
        run_noop,
        StateManager::new(),
    )
    .await
    .unwrap_err();

    assert!(error.to_string().contains("in the past"));
    scheduler.shutdown().await.unwrap();
}

#[tokio::test]
async fn adds_future_once_cron_and_calendar_jobs() {
    let mut scheduler = autopilot_core_time::init::init_time_check().await.unwrap();
    let future = chrono::Local::now() + chrono::Duration::days(1);
    let once = When::Once(autopilot_core_time::DateTimeScheme {
        date: future.format("%Y/%m/%d").to_string(),
        time: "12:00".to_string(),
    });

    add_job(
        &job("once", Some(once)),
        &scheduler,
        run_noop,
        StateManager::new(),
    )
    .await
    .unwrap();
    add_job(
        &job("cron", Some(When::Cron("0 0 * * * *".to_string()))),
        &scheduler,
        run_noop,
        StateManager::new(),
    )
    .await
    .unwrap();
    add_job(
        &job(
            "daily",
            Some(When::Daily(autopilot_core_time::TimeScheme {
                time: "12:00".to_string(),
            })),
        ),
        &scheduler,
        run_noop,
        StateManager::new(),
    )
    .await
    .unwrap();

    scheduler.shutdown().await.unwrap();
}

#[tokio::test]
async fn rejects_invalid_cron_expressions() {
    let mut scheduler = autopilot_core_time::init::init_time_check().await.unwrap();
    let error = add_job(
        &job("bad_cron", Some(When::Cron("not a cron".to_string()))),
        &scheduler,
        run_noop,
        StateManager::new(),
    )
    .await
    .unwrap_err();

    assert!(matches!(error, AutoPilotError::Cron(_)));
    scheduler.shutdown().await.unwrap();
}
