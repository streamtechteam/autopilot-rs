use super::*;
use std::sync::{Mutex, MutexGuard, Once};

static CONFIG_INIT: Once = Once::new();
static CONFIG_LOCK: Mutex<()> = Mutex::new(());

fn config_lock() -> MutexGuard<'static, ()> {
    CONFIG_INIT.call_once(|| {
        let dir = std::env::temp_dir().join(format!(
            "autopilot-api-spec-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        autopilot_core::fs::init(Some(dir)).unwrap();
    });

    CONFIG_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn job(id: &str, name: &str) -> autopilot_core::job::Job {
    autopilot_core::job::Job::new(
        id.to_string(),
        name.to_string(),
        "Description".to_string(),
        None,
        None,
        vec![],
        vec![],
    )
}

fn payload(id: &str, name: &str) -> JobScheme {
    JobScheme {
        id: id.to_string(),
        name: Some(name.to_string()),
        description: Some("description".to_string()),
        when: None,
        check_interval: None,
        conditions: vec![],
        tasks: vec![],
    }
}

async fn state_with_jobs(jobs: Vec<autopilot_core::job::Job>) -> AppState {
    let mut autopilot = autopilot_core::autopilot::AutoPilot::new().await.unwrap();
    autopilot.jobs = jobs;
    AppState {
        auto_pilot: std::sync::Arc::new(tokio::sync::RwLock::new(autopilot)),
        started: std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
    }
}

#[tokio::test]
async fn health_reports_ok() {
    let response = health().await.0;

    assert_eq!(response.status, "ok");
    assert_eq!(response.service, API_SERVICE_NAME);
}

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
    let status = autopilot_core::status::JobStatusStruct::new(
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

#[tokio::test]
async fn jobs_status_reports_count_and_running_flag() {
    let state = state_with_jobs(vec![job("job_1", "One"), job("job_2", "Two")]).await;

    let response = jobs_status(State(state.clone())).await.0;
    assert!(!response.running);
    assert_eq!(response.job_count, 2);

    state
        .started
        .store(true, std::sync::atomic::Ordering::Relaxed);
    let response = jobs_status(State(state)).await.0;
    assert!(response.running);
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
    state
        .started
        .store(true, std::sync::atomic::Ordering::Relaxed);

    let response = jobs_stop(State(state.clone())).await.unwrap().0;
    assert_eq!(response["success"], true);
    assert_eq!(response["message"], "Jobs stopped");
    assert!(!state.started.load(std::sync::atomic::Ordering::Relaxed));
}

#[tokio::test]
async fn jobs_getbyid_finds_loaded_jobs() {
    let state = state_with_jobs(vec![job("job_find", "Find")]).await;

    let response = jobs_getbyid(State(state.clone()), Path("job_find".to_string()))
        .await
        .unwrap()
        .0;
    assert_eq!(response.id, "job_find");
    assert_eq!(response.name, "Find");

    assert_eq!(
        jobs_getbyid(State(state), Path("missing".to_string()))
            .await
            .unwrap_err(),
        StatusCode::NOT_FOUND
    );
}

#[tokio::test]
async fn jobs_list_merges_live_statuses() {
    let _guard = config_lock();
    let state = state_with_jobs(vec![job("job_list_live", "List Live")]).await;
    {
        let autopilot = state.auto_pilot.read().await;
        autopilot
            .status_manager
            .set(
                "job_list_live".to_string(),
                Some("List Live".to_string()),
                JobStatusEnum::Running,
            )
            .await;
    }
    state
        .started
        .store(true, std::sync::atomic::Ordering::Relaxed);

    let response = jobs_list(State(state.clone())).await.unwrap().0;
    let listed = response
        .iter()
        .find(|job| job.id == "job_list_live")
        .unwrap();
    assert_eq!(listed.status, "Running");
    assert!(listed.loaded);

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
        None,
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
        Json(payload("ignored", "Ignored")),
    )
    .await
    .unwrap()
    .0;
    let new_path = response["path"].as_str().unwrap();

    assert_eq!(response["success"], true);
    assert!(!path.exists());
    assert!(std::path::Path::new(new_path).exists());

    std::fs::remove_file(new_path).unwrap();
}

#[tokio::test]
async fn jobs_update_reports_missing_jobs_as_server_errors() {
    let _guard = config_lock();
    let state = state_with_jobs(vec![]).await;

    let error = jobs_update(
        State(state),
        Path("job_does_not_exist".to_string()),
        Json(payload("ignored", "Ignored")),
    )
    .await
    .unwrap_err();

    assert_eq!(error, StatusCode::INTERNAL_SERVER_ERROR);
}
