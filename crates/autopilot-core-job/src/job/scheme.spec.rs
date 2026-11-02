use super::*;

fn scheme(check_interval: Option<&str>, when: Option<When>) -> JobScheme {
    JobScheme {
        id: "job_validate".to_string(),
        name: None,
        description: None,
        when,
        check_interval: check_interval.map(str::to_string),
        conditions: vec![],
        tasks: vec![],
        task_mode: TaskMode::default(),
    }
}

#[test]
fn check_intervals_must_be_numbers_of_at_least_100_ms() {
    assert_eq!(parse_check_interval("250").unwrap(), 250);
    assert_eq!(parse_check_interval(" 100 ").unwrap(), 100);

    for value in ["0", "99", "", "abc", "1.5", "-100"] {
        assert!(parse_check_interval(value).is_err(), "{value}");
    }
}

#[test]
fn validate_rejects_bad_intervals_and_cron_expressions() {
    assert!(scheme(Some("0"), None).validate().is_err());
    assert!(
        scheme(None, Some(When::Cron("* * * * *".to_string())))
            .validate()
            .is_err()
    );
    assert!(
        scheme(None, Some(When::Cron("*/5 * * * * *".to_string())))
            .validate()
            .is_ok()
    );
    assert!(scheme(None, None).validate().is_ok());
}

#[test]
fn validate_rejects_structured_schedules_with_invalid_values() {
    let invalid_weekly = When::Weekly(autopilot_core_time::WeeklyScheme {
        time: "25:00".to_string(),
        weekday: 1,
    });

    assert!(scheme(None, Some(invalid_weekly)).validate().is_err());
}
