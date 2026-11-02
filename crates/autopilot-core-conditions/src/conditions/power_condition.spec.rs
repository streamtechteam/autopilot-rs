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
    });

    assert!(!condition.check_charging);
    assert_eq!(condition.threshold, Some(75.0));
    assert_eq!(condition.operator.as_deref(), Some("greater"));
    assert_eq!(condition.name(), "Power");
    assert_eq!(condition.clone_box().name(), "Power");
}

#[cfg(target_os = "linux")]
#[test]
fn linux_power_scan_finds_renamed_ac_and_battery_entries() {
    let root = std::env::temp_dir().join(format!(
        "autopilot-power-spec-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(root.join("ADP1")).unwrap();
    std::fs::create_dir_all(root.join("BAT1")).unwrap();
    std::fs::write(root.join("ADP1/online"), "1\n").unwrap();
    std::fs::write(root.join("BAT1/type"), "Battery\n").unwrap();
    std::fs::write(root.join("BAT1/capacity"), "42\n").unwrap();

    assert!(any_ac_online(&root));
    assert_eq!(battery_capacity(&root), Some(42.0));

    std::fs::write(root.join("ADP1/online"), "0\n").unwrap();
    assert!(!any_ac_online(&root));

    std::fs::remove_dir_all(root).unwrap();
}
