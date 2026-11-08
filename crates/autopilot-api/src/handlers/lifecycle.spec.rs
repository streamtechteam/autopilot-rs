use axum::{extract::State, http::StatusCode};

use super::{jobs_reload, jobs_start, jobs_status, jobs_stop};
use crate::handlers::test_support::{config_lock, job, state_with_jobs};

#[tokio::test]
async fn jobs_status_reports_count_and_running_flag() {
    let state = state_with_jobs(vec![job("job_1", "One"), job("job_2", "Two")]).await;

    let response = jobs_status(State(state.clone())).await.0;
    assert!(!response.running);
    assert_eq!(response.job_count, 2);
    assert!(!response.reload_required);

    state.auto_pilot.write().await.mark_reload_required();
    let response = jobs_status(State(state.clone())).await.0;
    assert!(response.reload_required);

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

#[tokio::test]
async fn reload_clears_reload_required_after_success() {
    let _guard = config_lock().await;
    let state = state_with_jobs(vec![]).await;
    state.auto_pilot.write().await.mark_reload_required();

    let response = jobs_reload(State(state.clone())).await.unwrap().0;
    assert!(response.success);
    assert!(!state.auto_pilot.read().await.reload_required);
}

#[tokio::test]
async fn start_clears_reload_required() {
    let _guard = config_lock().await;
    let state = state_with_jobs(vec![]).await;
    state.auto_pilot.write().await.mark_reload_required();

    let response = jobs_start(State(state.clone())).await.unwrap().0;
    assert_eq!(response["success"], true);
    assert!(!state.auto_pilot.read().await.reload_required);

    state.auto_pilot.write().await.shutdown().await.unwrap();
}

#[tokio::test]
async fn reload_returns_500_when_jobs_cannot_be_loaded() {
    let _guard = config_lock().await;
    let state = state_with_jobs(vec![]).await;
    state.auto_pilot.write().await.mark_reload_required();
    let jobs_dir = autopilot_core::fs::config::current().unwrap().jobs_dir();
    std::fs::create_dir_all(&jobs_dir).unwrap();
    let invalid = jobs_dir.join("invalid_reload_spec.jsonc");
    std::fs::write(&invalid, "{ not valid json").unwrap();

    let (status, body) = jobs_reload(State(state.clone())).await.unwrap_err();
    assert_eq!(status, StatusCode::INTERNAL_SERVER_ERROR);
    assert!(!body.0.success);
    assert!(state.auto_pilot.read().await.reload_required);

    std::fs::remove_file(invalid).unwrap();
}
