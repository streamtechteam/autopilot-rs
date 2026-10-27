use super::*;

#[test]
fn constructors_set_charging_and_battery_modes() {
    let charging = PowerCondition::is_charging();
    let battery = PowerCondition::battery_level(50.0, "LESS".to_string());

    assert!(charging.check_charging);
    assert_eq!(charging.threshold, None);
    assert!(!battery.check_charging);
    assert_eq!(battery.threshold, Some(50.0));
    assert_eq!(battery.operator.as_deref(), Some("less"));
}

#[test]
fn from_scheme_defaults_and_lowercases_operator() {
    let condition = PowerCondition::from_scheme(PowerConditionScheme {
        check_charging: None,
        threshold: Some(75.0),
        operator: Some("GREATER".to_string()),
        check_interval: None,
    });

    assert!(!condition.check_charging);
    assert_eq!(condition.threshold, Some(75.0));
    assert_eq!(condition.operator.as_deref(), Some("greater"));
    assert_eq!(condition.name(), "Power");
    assert_eq!(condition.clone_box().name(), "Power");
}
