use axum::{
    Json,
    extract::{Path, State},
    http::StatusCode,
};

use autopilot_core::job::set::add_job;
use autopilot_core::status::JobStatusEnum;

use super::{jobs_create, jobs_delete, jobs_getbyid, jobs_list, jobs_update};
use crate::handlers::test_support::{config_lock, payload, state_with_jobs};

#[tokio::test]
async fn jobs_getbyid_finds_jobs_on_disk() {
    let _guard = config_lock();
    let path = add_job(
        Some("Find".to_string()),
        Some("Find description".to_string()),
        None,
        None,
        vec![],
        vec![],
    )
    .unwrap();
    let id = autopilot_core::job::get::get_job(path.clone()).unwrap().id;
    let state = state_with_jobs(vec![]).await;

    let response = jobs_getbyid(State(state.clone()), Path(id.clone()))
        .await
        .unwrap()
        .0;
    assert_eq!(response.id, id);
    assert_eq!(response.name, "Find");
    assert_eq!(response.description, "Find description");

    assert_eq!(
        jobs_getbyid(State(state), Path("missing".to_string()))
            .await
            .unwrap_err(),
        StatusCode::NOT_FOUND
    );

    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn jobs_list_merges_live_statuses() {
    let _guard = config_lock();
    let path = add_job(
        Some("List Live".to_string()),
        Some("List description".to_string()),
        None,
        None,
        vec![],
        vec![],
    )
    .unwrap();
    let id = autopilot_core::job::get::get_job(path.clone()).unwrap().id;
    let state = state_with_jobs(vec![]).await;
    {
        let autopilot = state.auto_pilot.read().await;
        autopilot
            .status_manager
            .set(
                id.clone(),
                Some("List Live".to_string()),
                JobStatusEnum::Running,
            )
            .await;
    }

    let response = jobs_list(State(state.clone())).await.unwrap().0;
    let listed = response.iter().find(|job| job.id == id).unwrap();
    assert_eq!(listed.status, "Running");
    assert_eq!(listed.description, "List description");
    assert!(listed.loaded);

    std::fs::remove_file(path).unwrap();
    state.auto_pilot.write().await.shutdown().await.unwrap();
}

#[tokio::test]
async fn jobs_create_writes_a_job_file() {
    let _guard = config_lock();
    let response = jobs_create(Json(payload("job_create_spec", "Create Spec")))
        .await
        .unwrap()
        .0;
    let path = response["path"].as_str().unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(response["message"], "Job created");
    assert!(std::path::Path::new(path).exists());

    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn jobs_delete_removes_a_job_file() {
    let _guard = config_lock();
    let path = add_job(
        Some("Delete Spec".to_string()),
        None,
        None,
        None,
        vec![],
        vec![],
    )
    .unwrap();
    let job = autopilot_core::job::get::get_job(path.clone()).unwrap();

    let response = jobs_delete(Path(job.id)).await.unwrap().0;
    assert_eq!(response["success"], true);
    assert!(!path.exists());
}

#[tokio::test]
async fn jobs_delete_reports_unknown_ids() {
    let _guard = config_lock();

    assert_eq!(
        jobs_delete(Path("job_does_not_exist".to_string()))
            .await
            .unwrap_err(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn jobs_update_replaces_an_existing_job() {
    let _guard = config_lock();
    let path = add_job(
        Some("Update Spec".to_string()),
        Some("before".to_string()),
        None,
        None,
        vec![],
        vec![],
    )
    .unwrap();
    let id = autopilot_core::job::get::get_job(path.clone()).unwrap().id;
    let state = state_with_jobs(vec![]).await;

    let response = jobs_update(
        State(state),
        Path(id.clone()),
        Json(payload("ignored", "Updated Spec")),
    )
    .await
    .unwrap()
    .0;
    let updated_path = response["path"].as_str().unwrap();

    assert_eq!(response["success"], true);
    assert_eq!(std::path::Path::new(updated_path), path.as_path());
    assert!(path.exists());

    let updated = autopilot_core::job::get::get_job(path.clone()).unwrap();
    assert_eq!(updated.id, id);
    assert_eq!(updated.name, "Updated Spec");
    assert_eq!(updated.description, "description");

    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn jobs_update_reports_missing_jobs_as_not_found() {
    let _guard = config_lock();
    let state = state_with_jobs(vec![]).await;

    let error = jobs_update(
        State(state),
        Path("job_does_not_exist".to_string()),
        Json(payload("ignored", "Ignored")),
    )
    .await
    .unwrap_err();

    assert_eq!(error, StatusCode::NOT_FOUND);
}
