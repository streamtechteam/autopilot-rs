use axum::{extract::State, http::StatusCode};

use super::{jobs_status, jobs_stop};
use crate::handlers::test_support::{job, state_with_jobs};

#[tokio::test]
async fn jobs_status_reports_count_and_running_flag() {
    let state = state_with_jobs(vec![job("job_1", "One"), job("job_2", "Two")]).await;

    let response = jobs_status(State(state.clone())).await.0;
    assert!(!response.running);
    assert_eq!(response.job_count, 2);

    state.auto_pilot.write().await.started = true;
    let response = jobs_status(State(state.clone())).await.0;
    assert!(response.running);

    state.auto_pilot.write().await.shutdown().await.unwrap();
}

#[tokio::test]
async fn jobs_stop_reports_conflict_when_already_stopped() {
    let state = state_with_jobs(vec![]).await;

    assert_eq!(
        jobs_stop(State(state)).await.unwrap_err(),
        StatusCode::CONFLICT
    );
}

#[tokio::test]
async fn jobs_stop_stops_a_started_instance() {
    let state = state_with_jobs(vec![]).await;
    state.auto_pilot.write().await.started = true;

    let response = jobs_stop(State(state.clone())).await.unwrap().0;
    assert_eq!(response["success"], true);
    assert_eq!(response["message"], "Jobs stopped");
    assert!(!state.auto_pilot.read().await.started);
}
