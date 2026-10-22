use autopilot_core::{job::Job, time::When};

fn job_json(extra: &str) -> String {
    format!(
        r#"{{
            "id": "job_test",
            "name": "test",
            "description": "test job",
            "check_interval": "500",
            "conditions": [{{"type": "command", "condition": {{"command": "true"}}}}],
            "tasks": [{{"command": "echo hello"}}]
            {extra}
        }}"#
    )
}

#[test]
fn parses_regular_job() {
    let job = Job::from_json(&job_json("")).expect("job should parse");

    assert_eq!(job.id, "job_test");
    assert_eq!(job.name, "test");
    assert_eq!(job.description, "test job");
    assert_eq!(job.check_interval.as_deref(), Some("500"));
    assert_eq!(job.conditions.len(), 1);
    assert_eq!(job.tasks.len(), 1);
    assert!(job.when.is_none());
}

#[test]
fn parses_cron_trigger() {
    let json = job_json(r#", "when": {"type": "cron", "trigger": "*/5 * * * * *"}"#);
    let job = Job::from_json(&json).expect("job should parse");

    match job.when {
        Some(When::Cron(expression)) => assert_eq!(expression, "*/5 * * * * *"),
        other => panic!("expected cron trigger, got {other:?}"),
    }
}

#[test]
fn rejects_invalid_json() {
    assert!(Job::from_json("{ not json").is_err());
}

#[test]
fn rejects_unknown_fields() {
    let json = job_json(r#", "unexpected": true"#);
    assert!(Job::from_json(&json).is_err());
}
