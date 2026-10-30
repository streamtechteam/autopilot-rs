use super::*;
use crate::job::test_support;
use autopilot_core_conditions::ConditionScheme;
use autopilot_core_conditions::fail_condition::FailCondition;
use autopilot_core_status::{JobStatusEnum, StateManager};
use autopilot_core_task::TaskScheme;

fn scheme(id: &str) -> JobScheme {
    JobScheme {
        id: id.to_string(),
        name: None,
        description: None,
        when: None,
        check_interval: None,
        conditions: vec![ConditionScheme::Fail(FailCondition)],
        tasks: vec![TaskScheme::new("true".to_string())],
        task_mode: TaskMode::Concurrent,
    }
}

#[test]
fn from_scheme_applies_defaults() {
    let job = Job::from_scheme(scheme("job_defaults"));

    assert_eq!(job.id, "job_defaults");
    assert_eq!(job.name, "job_job_defaults");
    assert_eq!(job.description, " ");
    assert!(!job.loaded);
    assert_eq!(job.conditions.len(), 1);
    assert_eq!(job.tasks.len(), 1);
    assert_eq!(job.tasks[0].command, "true");
    assert_eq!(job.task_mode, TaskMode::Concurrent);
}

#[test]
fn from_scheme_preserves_optional_fields() {
    let mut input = scheme("job_options");
    input.name = Some("Named Job".to_string());
    input.description = Some("Description".to_string());
    input.check_interval = Some("250".to_string());
    let job = Job::from_scheme(input);

    assert_eq!(job.name, "Named Job");
    assert_eq!(job.description, "Description");
    assert_eq!(job.check_interval.as_deref(), Some("250"));
}

#[test]
fn task_mode_defaults_to_concurrent() {
    let job = Job::from_scheme(scheme("job_task_mode_default"));

    assert_eq!(job.task_mode, TaskMode::Concurrent);
}

#[test]
fn task_mode_deserializes_from_job_json() {
    let concurrent =
        Job::from_json(r#"{"id":"a","conditions":[],"tasks":[],"task_mode":"concurrent"}"#)
            .unwrap();
    let sequential =
        Job::from_json(r#"{"id":"b","conditions":[],"tasks":[],"task_mode":"sequential"}"#)
            .unwrap();
    let stop = Job::from_json(
        r#"{"id":"c","conditions":[],"tasks":[],"task_mode":"sequential_stop_on_failure"}"#,
    )
    .unwrap();

    assert_eq!(concurrent.task_mode, TaskMode::Concurrent);
    assert_eq!(sequential.task_mode, TaskMode::Sequential);
    assert_eq!(stop.task_mode, TaskMode::SequentialStopOnFailure);
}

#[test]
fn from_json_parses_and_rejects_invalid_json() {
    let json = serde_json::to_string(&scheme("job_json")).unwrap();
    let job = Job::from_json(&json).unwrap();

    assert_eq!(job.id, "job_json");
    assert!(Job::from_json("not json").is_err());
    assert!(Job::from_json(r#"{"id":"broken"}"#).is_err());
}

#[test]
fn add_condition_appends_to_the_list() {
    let mut job = Job::from_scheme(scheme("job_add_condition"));
    job.add_condition(Box::new(FailCondition));

    assert_eq!(job.conditions.len(), 2);
}

#[tokio::test]
async fn run_completes_when_all_conditions_pass() {
    let mut job = Job::from_scheme(scheme("job_run_completed"));
    job.conditions.clear();
    let state = StateManager::new();
    let mut scheduler = autopilot_core_time::init::init_time_check().await.unwrap();

    job.run(state.clone(), &scheduler, true).await;

    assert_eq!(
        state.get_state_by_id("job_run_completed".to_string()).await,
        Some(JobStatusEnum::Completed)
    );
    scheduler.shutdown().await.unwrap();
}

#[tokio::test]
async fn run_marks_unsatisfied_without_retry_interval() {
    let job = Job::from_scheme(scheme("job_run_unsatisfied"));
    let state = StateManager::new();
    let mut scheduler = autopilot_core_time::init::init_time_check().await.unwrap();

    job.clone().run(state.clone(), &scheduler, true).await;

    assert_eq!(
        state
            .get_state_by_id("job_run_unsatisfied".to_string())
            .await,
        Some(JobStatusEnum::Unsatisfied)
    );
    scheduler.shutdown().await.unwrap();
}

struct EventuallyTrue(std::sync::Arc<std::sync::atomic::AtomicUsize>);

impl Condition for EventuallyTrue {
    fn check(&self) -> bool {
        self.0.fetch_add(1, std::sync::atomic::Ordering::SeqCst) > 0
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(Self(self.0.clone()))
    }

    fn name(&self) -> &str {
        "EventuallyTrue"
    }
}

#[tokio::test]
async fn run_retries_until_the_condition_passes() {
    let mut input = scheme("job_run_retry");
    input.conditions.clear();
    input.check_interval = Some("10".to_string());
    let mut job = Job::from_scheme(input);
    let checks = std::sync::Arc::new(std::sync::atomic::AtomicUsize::new(0));
    job.conditions
        .push(Box::new(EventuallyTrue(checks.clone())));
    let state = StateManager::new();
    let mut scheduler = autopilot_core_time::init::init_time_check().await.unwrap();

    job.run(state.clone(), &scheduler, true).await;

    assert!(checks.load(std::sync::atomic::Ordering::SeqCst) >= 2);
    assert_eq!(
        state.get_state_by_id("job_run_retry".to_string()).await,
        Some(JobStatusEnum::Completed)
    );
    scheduler.shutdown().await.unwrap();
}

#[tokio::test]
async fn run_job_marks_completed_or_unsatisfied() {
    let state = StateManager::new();
    let job = Job::from_scheme(scheme("job_run_job"));
    run_job(job, state.clone()).await;
    assert_eq!(
        state.get_state_by_id("job_run_job".to_string()).await,
        Some(JobStatusEnum::Unsatisfied)
    );

    let mut passing = scheme("job_run_job_pass");
    passing.conditions.clear();
    run_job(Job::from_scheme(passing), state.clone()).await;
    assert_eq!(
        state.get_state_by_id("job_run_job_pass".to_string()).await,
        Some(JobStatusEnum::Completed)
    );

    let mut failing = scheme("job_run_job_fail");
    failing.conditions.clear();
    failing.tasks = vec![TaskScheme::new("exit 1".to_string())];
    run_job(Job::from_scheme(failing), state.clone()).await;
    assert_eq!(
        state.get_state_by_id("job_run_job_fail".to_string()).await,
        Some(JobStatusEnum::Failed)
    );
}

struct PanickingCondition;

impl Condition for PanickingCondition {
    fn check(&self) -> bool {
        panic!("condition panic");
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(PanickingCondition)
    }

    fn name(&self) -> &str {
        "Panicking"
    }
}

#[tokio::test]
async fn run_job_treats_a_panicking_condition_as_unsatisfied() {
    let state = StateManager::new();
    let job = Job::new(
        "job_panicking_condition".to_string(),
        "job_panicking_condition".to_string(),
        " ".to_string(),
        None,
        None,
        vec![Box::new(PanickingCondition)],
        Vec::new(),
        TaskMode::Concurrent,
    );

    run_job(job, state.clone()).await;

    assert_eq!(
        state
            .get_state_by_id("job_panicking_condition".to_string())
            .await,
        Some(JobStatusEnum::Unsatisfied)
    );
}

#[tokio::test]
async fn run_tasks_reports_individual_failures_but_runs_all_concurrent_tasks() {
    let tasks = vec![
        autopilot_core_task::Task::new("exit 0".to_string()),
        autopilot_core_task::Task::new("exit 1".to_string()),
    ];

    assert!(!run_tasks(&tasks, TaskMode::Concurrent).await);
}

fn sequential_test_file() -> std::path::PathBuf {
    let unique = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!(
        "autopilot-sequential-{}-{unique}",
        std::process::id()
    ))
}

fn create_file_command(path: &std::path::Path) -> String {
    if cfg!(windows) {
        format!("type nul > \"{}\"", path.display())
    } else {
        format!("touch '{}'", path.display())
    }
}

#[tokio::test]
async fn sequential_task_mode_continues_after_failure() {
    let path = sequential_test_file();
    let _ = std::fs::remove_file(&path);
    let tasks = vec![
        autopilot_core_task::Task::new("exit 1".to_string()),
        autopilot_core_task::Task::new(create_file_command(&path)),
    ];

    assert!(!run_tasks(&tasks, TaskMode::Sequential).await);

    assert!(path.exists());
    std::fs::remove_file(path).unwrap();
}

#[tokio::test]
async fn sequential_stop_on_failure_skips_remaining_tasks() {
    let path = sequential_test_file();
    let _ = std::fs::remove_file(&path);
    let tasks = vec![
        autopilot_core_task::Task::new("exit 1".to_string()),
        autopilot_core_task::Task::new(create_file_command(&path)),
    ];

    assert!(!run_tasks(&tasks, TaskMode::SequentialStopOnFailure).await);

    assert!(!path.exists());
}

#[tokio::test]
async fn load_jobs_reads_the_config_directory() {
    let _guard = test_support::lock();
    let path = test_support::jobs_dir().join("job_load_spec.jsonc");
    std::fs::write(
        &path,
        r#"{"id":"job_load_spec","name":"Load Spec","conditions":[],"tasks":[]}"#,
    )
    .unwrap();

    let jobs = crate::job::get::get_jobs(false).unwrap();
    assert!(jobs.iter().any(|job| job.id == "job_load_spec"));

    std::fs::remove_file(path).unwrap();
}

// #[test]
// fn committed_schema_matches_job_scheme() {
//     let generated = serde_json::to_string_pretty(&schemars::schema_for!(JobScheme)).unwrap();
//     let schema_path =
//         std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../json_schema/schema.json");
//     let committed = std::fs::read_to_string(schema_path).unwrap();

//     assert_eq!(committed, format!("{generated}\n"));
// }

#[test]
fn bundled_job_template_matches_scheme() {
    let template_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/job.jsonc");
    let template = std::fs::read_to_string(template_path).unwrap();
    let parsed = autopilot_core_fs::jsonc::parse(&template);

    let job: JobScheme = serde_json::from_str(&parsed).unwrap();

    assert_eq!(job.id, "example-job");
    assert_eq!(job.tasks.len(), 1);
}
