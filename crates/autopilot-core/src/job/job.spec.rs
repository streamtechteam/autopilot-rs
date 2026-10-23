use super::*;
use crate::conditions::ConditionScheme;
use crate::conditions::fail_condition::FailCondition;
use crate::fs::test_support;
use crate::task::TaskScheme;

fn scheme(id: &str) -> JobScheme {
    JobScheme {
        id: id.to_string(),
        name: None,
        description: None,
        when: None,
        check_interval: None,
        conditions: vec![ConditionScheme::Fail(FailCondition)],
        tasks: vec![TaskScheme::new("true".to_string())],
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
    let mut scheduler = crate::time::init::init_time_check().await.unwrap();

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
    let mut scheduler = crate::time::init::init_time_check().await.unwrap();

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
    let mut scheduler = crate::time::init::init_time_check().await.unwrap();

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
}

#[tokio::test]
async fn run_tasks_ignores_individual_failures() {
    let tasks = vec![
        crate::task::Task::new("exit 0".to_string()),
        crate::task::Task::new("exit 1".to_string()),
    ];

    run_tasks(&tasks).await;
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

    let mut autopilot = crate::autopilot::AutoPilot::new().await.unwrap();
    autopilot.load_jobs().await.unwrap();

    assert!(autopilot.jobs.iter().any(|job| job.id == "job_load_spec"));

    std::fs::remove_file(path).unwrap();
    autopilot.shutdown().await.unwrap();
}

#[test]
fn committed_schema_matches_job_scheme() {
    let generated = serde_json::to_string_pretty(&schemars::schema_for!(JobScheme)).unwrap();
    let schema_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../json_schema/schema.json");
    let committed = std::fs::read_to_string(schema_path).unwrap();

    assert_eq!(committed, format!("{generated}\n"));
}

#[test]
fn bundled_job_template_matches_scheme() {
    let template_path =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../templates/job.jsonc");
    let template = std::fs::read_to_string(template_path).unwrap();
    let parsed = crate::utilities::jsonc_parser::jsonc_parse(&template);

    let job: JobScheme = serde_json::from_str(&parsed).unwrap();

    assert_eq!(job.id, "example-job");
    assert_eq!(job.tasks.len(), 1);
}
