use autopilot_core::status::{JobStatusEnum, JobStatusStruct, StateManager};

use super::JobResponse;
use crate::handlers::test_support::job;

#[tokio::test]
async fn job_response_uses_the_live_status() {
    let job = job("job_live", "Live");
    let state = StateManager::new();

    let response = JobResponse::from_job_and_sm(&job, state.clone()).await;
    assert_eq!(response.id, "job_live");
    assert_eq!(response.name, "Live");
    assert_eq!(response.description, "Description");
    assert!(!response.loaded);
    assert_eq!(response.status, "None");

    state
        .set("job_live".to_string(), None, JobStatusEnum::Running)
        .await;
    let response = JobResponse::from_job_and_sm(&job, state).await;
    assert_eq!(response.status, "Some(Running)");
}

#[test]
fn job_response_converts_from_status_struct() {
    let status = JobStatusStruct::new(
        "job_status".to_string(),
        "Status".to_string(),
        JobStatusEnum::Failed,
    );

    let response = JobResponse::from(&status);
    assert_eq!(response.id, "job_status");
    assert_eq!(response.name, "Status");
    assert_eq!(response.description, "Status");
    assert!(response.loaded);
    assert_eq!(response.status, "Failed");
}
