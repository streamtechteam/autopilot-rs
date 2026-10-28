use super::*;
use autopilot_core::job::TaskMode;

#[test]
fn formats_known_job_options() {
    let job = JobScheme {
        id: "job_123_Backup".to_string(),
        name: Some("Backup".to_string()),
        description: None,
        when: None,
        check_interval: None,
        conditions: Vec::new(),
        tasks: Vec::new(),
        task_mode: TaskMode::Concurrent,
    };

    assert_eq!(format_job_option(&job), "Backup - job_123_Backup");
}

#[test]
fn formats_unknown_job_options() {
    let job = JobScheme {
        id: "broken".to_string(),
        name: None,
        description: None,
        when: None,
        check_interval: None,
        conditions: Vec::new(),
        tasks: Vec::new(),
        task_mode: TaskMode::Concurrent,
    };

    assert_eq!(format_job_option(&job), "Unknown - broken");
}
