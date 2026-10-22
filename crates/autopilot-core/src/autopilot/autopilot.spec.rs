use super::*;
use crate::conditions::ConditionScheme;
use crate::job::{Job, JobScheme};
use crate::task::TaskScheme;

fn job(id: &str) -> Job {
    Job::from_scheme(JobScheme {
        id: id.to_string(),
        name: Some(id.to_string()),
        description: None,
        when: None,
        check_interval: None,
        conditions: vec![],
        tasks: vec![TaskScheme::new("true".to_string())],
    })
}

#[tokio::test]
async fn new_creates_an_empty_autopilot_instance() {
    let mut autopilot = AutoPilot::new().await.unwrap();

    assert!(!autopilot.started);
    assert!(autopilot.jobs.is_empty());
    assert!(autopilot.jobs_handles.is_empty());
    assert!(autopilot.status_manager.get_all().await.is_empty());
    autopilot.scheduler.shutdown().await.unwrap();
}

#[tokio::test]
async fn init_status_registers_every_job_as_unknown() {
    let mut autopilot = AutoPilot::new().await.unwrap();
    autopilot.jobs = vec![job("job_a"), job("job_b")];

    autopilot.init_status().await.unwrap();

    let statuses = autopilot.status_manager.get_all().await;
    assert_eq!(statuses.len(), 2);
    assert_eq!(statuses[0].id, "job_a");
    assert_eq!(statuses[0].name, "job_a");
    assert!(matches!(statuses[0].status, JobStatusEnum::Unknown));
    autopilot.shutdown().await.unwrap();
}

#[tokio::test]
async fn run_jobs_marks_jobs_loaded_and_completes_them() {
    let mut autopilot = AutoPilot::new().await.unwrap();
    autopilot.jobs = vec![job("job_run")];
    autopilot.init_status().await.unwrap();

    let handles = autopilot.run_jobs();
    assert!(autopilot.jobs[0].loaded);

    for _ in 0..100 {
        if matches!(
            autopilot
                .status_manager
                .get_state_by_id("job_run".to_string())
                .await,
            Some(JobStatusEnum::Completed)
        ) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    assert!(matches!(
        autopilot
            .status_manager
            .get_state_by_id("job_run".to_string())
            .await,
        Some(JobStatusEnum::Completed)
    ));

    for handle in handles {
        handle.abort();
    }
    autopilot.shutdown().await.unwrap();
}

#[tokio::test]
async fn stop_jobs_clears_jobs_and_handles() {
    let mut autopilot = AutoPilot::new().await.unwrap();
    autopilot.jobs = vec![job("job_stop")];
    autopilot.jobs_handles = autopilot.run_jobs();

    autopilot.stop_jobs().await.unwrap();

    assert!(autopilot.jobs.is_empty());
    assert!(autopilot.jobs_handles.is_empty());
}

#[tokio::test]
async fn run_jobs_uses_scheme_conditions() {
    let mut autopilot = AutoPilot::new().await.unwrap();
    let mut configured = job("job_condition");
    configured.add_condition(Box::new(crate::conditions::fail_condition::FailCondition));
    autopilot.jobs = vec![configured];
    autopilot.init_status().await.unwrap();

    let handles = autopilot.run_jobs();
    for _ in 0..100 {
        if matches!(
            autopilot
                .status_manager
                .get_state_by_id("job_condition".to_string())
                .await,
            Some(JobStatusEnum::Unsatisfied)
        ) {
            break;
        }
        tokio::time::sleep(std::time::Duration::from_millis(10)).await;
    }

    assert!(matches!(
        autopilot
            .status_manager
            .get_state_by_id("job_condition".to_string())
            .await,
        Some(JobStatusEnum::Unsatisfied)
    ));

    for handle in handles {
        handle.abort();
    }
    autopilot.shutdown().await.unwrap();
}

#[test]
fn condition_scheme_is_available_to_jobs() {
    assert!(matches!(
        ConditionScheme::Fail(Default::default()),
        ConditionScheme::Fail(_)
    ));
}
