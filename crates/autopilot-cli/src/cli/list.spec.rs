use super::*;
use autopilot_core::job::TaskMode;

fn job(id: &str, name: &str) -> JobScheme {
    JobScheme {
        id: id.to_string(),
        name: Some(name.to_string()),
        description: None,
        when: None,
        check_interval: None,
        conditions: Vec::new(),
        tasks: Vec::new(),
        task_mode: TaskMode::Concurrent,
    }
}

#[test]
fn formats_a_single_job() {
    assert_eq!(format_job(&job("job_1", "Backup")), "job_1 - Backup");
}

#[test]
fn formats_a_nameless_job_with_its_id() {
    let mut job = job("job_1", "Backup");
    job.name = None;

    assert_eq!(format_job(&job), "job_1 - job_1");
}

#[test]
fn formats_all_jobs_in_order() {
    let jobs = vec![job("job_1", "One"), job("job_2", "Two")];

    assert_eq!(
        format_jobs(&jobs),
        vec!["job_1 - One".to_string(), "job_2 - Two".to_string()]
    );
    assert!(format_jobs(&[]).is_empty());
}
