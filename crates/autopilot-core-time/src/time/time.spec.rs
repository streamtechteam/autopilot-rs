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
        to_cron_expression(When::Weekly(WeeklyScheme {
            time: "18:30".to_string(),
            weekday: 1,
        }))
        .unwrap(),
        "0 30 18 * * 1"
    );
    assert_eq!(
        to_cron_expression(When::Monthly(MonthlyScheme {
            time: "01:02".to_string(),
            day: 1,
        }))
        .unwrap(),
        "0 2 1 1 * *"
    );
    assert_eq!(
        to_cron_expression(When::Yearly(YearlyScheme {
            time: "23:59".to_string(),
            month: 1,
            day: 1,
        }))
        .unwrap(),
        "0 59 23 1 1 *"
    );
}

#[test]
fn builds_cron_expressions_for_custom_dates() {
    assert_eq!(
        to_cron_expression(When::Weekly(WeeklyScheme {
            time: "18:30".to_string(),
            weekday: 5,
        }))
        .unwrap(),
        "0 30 18 * * 5"
    );
    assert_eq!(
        to_cron_expression(When::Weekly(WeeklyScheme {
            time: "18:30".to_string(),
            weekday: 7,
        }))
        .unwrap(),
        "0 30 18 * * 0"
    );
    assert_eq!(
        to_cron_expression(When::Monthly(MonthlyScheme {
            time: "01:02".to_string(),
            day: 15,
        }))
        .unwrap(),
        "0 2 1 15 * *"
    );
    assert_eq!(
        to_cron_expression(When::Yearly(YearlyScheme {
            time: "23:59".to_string(),
            month: 6,
            day: 20,
        }))
        .unwrap(),
        "0 59 23 20 6 *"
    );
}

#[test]
fn rejects_out_of_range_schedule_fields() {
    assert!(
        to_cron_expression(When::Weekly(WeeklyScheme {
            time: "18:30".to_string(),
            weekday: 8,
        }))
        .is_err()
    );
    assert!(
        to_cron_expression(When::Monthly(MonthlyScheme {
            time: "01:02".to_string(),
            day: 0,
        }))
        .is_err()
    );
    assert!(
        to_cron_expression(When::Monthly(MonthlyScheme {
            time: "01:02".to_string(),
            day: 32,
        }))
        .is_err()
    );
    assert!(
        to_cron_expression(When::Yearly(YearlyScheme {
            time: "23:59".to_string(),
            month: 13,
            day: 1,
        }))
        .is_err()
    );
    assert!(
        to_cron_expression(When::Yearly(YearlyScheme {
            time: "23:59".to_string(),
            month: 1,
            day: 0,
        }))
        .is_err()
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

#[test]
fn recurring_schedule_fields_are_optional_and_default_to_existing_behavior() {
    let weekly: When =
        serde_json::from_str(r#"{"type":"weekly","trigger":{"time":"18:30"}}"#).unwrap();
    let monthly: When =
        serde_json::from_str(r#"{"type":"monthly","trigger":{"time":"01:02"}}"#).unwrap();
    let yearly: When =
        serde_json::from_str(r#"{"type":"yearly","trigger":{"time":"23:59"}}"#).unwrap();

    assert!(matches!(
        weekly,
        When::Weekly(WeeklyScheme { weekday: 1, .. })
    ));
    assert!(matches!(
        monthly,
        When::Monthly(MonthlyScheme { day: 1, .. })
    ));
    assert!(matches!(
        yearly,
        When::Yearly(YearlyScheme {
            month: 1,
            day: 1,
            ..
        })
    ));
}

#[test]
fn rejects_unknown_schedule_fields() {
    let unknown_schedules = [
        r#"{"type":"once","trigger":{"date":"2030/01/02","time":"03:04","zone":"UTC"}}"#,
        r#"{"type":"daily","trigger":{"time":"03:04","interval":1}}"#,
        r#"{"type":"weekly","trigger":{"time":"18:30","weekdays":5}}"#,
        r#"{"type":"monthly","trigger":{"time":"01:02","day_of_month":15}}"#,
        r#"{"type":"yearly","trigger":{"time":"23:59","month":6,"day":20,"year":2030}}"#,
    ];

    for schedule in unknown_schedules {
        assert!(
            serde_json::from_str::<When>(schedule).is_err(),
            "schedule accepted an unknown field: {schedule}"
        );
    }
}

#[test]
fn recurring_schedule_fields_round_trip() {
    let weekly = When::Weekly(WeeklyScheme {
        time: "18:30".to_string(),
        weekday: 5,
    });
    let monthly = When::Monthly(MonthlyScheme {
        time: "01:02".to_string(),
        day: 15,
    });
    let yearly = When::Yearly(YearlyScheme {
        time: "23:59".to_string(),
        month: 6,
        day: 20,
    });

    for schedule in [weekly, monthly, yearly] {
        let json = serde_json::to_string(&schedule).unwrap();
        let parsed: When = serde_json::from_str(&json).unwrap();
        assert_eq!(
            serde_json::to_string(&parsed).unwrap(),
            json,
            "schedule did not round-trip: {json}"
        );
    }
}
