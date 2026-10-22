use super::*;

#[test]
fn parses_local_date_and_time() {
    let scheme = DateTimeScheme {
        date: "2030/01/02".to_string(),
        time: "03:04".to_string(),
    };
    let parsed = scheme.parse().unwrap();

    assert_eq!(
        parsed.format("%Y/%m/%d %H:%M").to_string(),
        "2030/01/02 03:04"
    );
}

#[test]
fn rejects_invalid_date_or_time() {
    let invalid_date = DateTimeScheme {
        date: "2030-01-02".to_string(),
        time: "03:04".to_string(),
    };
    let invalid_time = DateTimeScheme {
        date: "2030/01/02".to_string(),
        time: "25:00".to_string(),
    };

    assert!(invalid_date.parse().is_err());
    assert!(invalid_time.parse().is_err());
}

#[test]
fn parses_time_scheme() {
    assert_eq!(
        TimeScheme {
            time: "09:05".to_string()
        }
        .parse()
        .unwrap()
        .to_string(),
        "09:05:00"
    );
    assert!(
        TimeScheme {
            time: "bad".to_string()
        }
        .parse()
        .is_err()
    );
}

#[test]
fn builds_cron_expressions_for_recurring_schedules() {
    assert_eq!(
        to_cron_expression(When::Daily(TimeScheme {
            time: "09:05".to_string()
        }))
        .unwrap(),
        "0 5 9 * * *"
    );
    assert_eq!(
        to_cron_expression(When::Weekly(TimeScheme {
            time: "18:30".to_string()
        }))
        .unwrap(),
        "0 30 18 * * 1"
    );
    assert_eq!(
        to_cron_expression(When::Monthly(TimeScheme {
            time: "01:02".to_string()
        }))
        .unwrap(),
        "0 2 1 1 * *"
    );
    assert_eq!(
        to_cron_expression(When::Yearly(TimeScheme {
            time: "23:59".to_string()
        }))
        .unwrap(),
        "0 59 23 1 1 *"
    );
}

#[test]
fn returns_empty_cron_for_once_and_cron_and_propagates_parse_errors() {
    assert_eq!(
        to_cron_expression(When::Cron("0 0 * * * *".to_string())).unwrap(),
        ""
    );
    assert!(
        to_cron_expression(When::Daily(TimeScheme {
            time: "invalid".to_string()
        }))
        .is_err()
    );
}

#[test]
fn when_uses_tagged_serde_representation() {
    let once: When =
        serde_json::from_str(r#"{"type":"once","trigger":{"date":"2030/01/02","time":"03:04"}}"#)
            .unwrap();
    let daily: When =
        serde_json::from_str(r#"{"type":"daily","trigger":{"time":"03:04"}}"#).unwrap();
    let cron: When = serde_json::from_str(r#"{"type":"cron","trigger":"0 0 * * * *"}"#).unwrap();

    assert!(matches!(once, When::Once(_)));
    assert!(matches!(daily, When::Daily(_)));
    assert!(matches!(cron, When::Cron(value) if value == "0 0 * * * *"));
}
