use super::*;

#[test]
fn from_scheme_preserves_screen_values() {
    let condition = ScreenCondition::from_scheme(ScreenConditionScheme {
        screen_count: Some(2),
        active_screen_name: Some("DP-1".to_string()),
        screen_names: Some(vec!["HDMI-1".to_string()]),
        check_interval: None,
    });

    assert_eq!(condition.screen_count, Some(2));
    assert_eq!(condition.active_screen_name.as_deref(), Some("DP-1"));
    assert_eq!(
        condition.screen_names.as_deref(),
        Some(["HDMI-1".to_string()].as_slice())
    );
    assert_eq!(condition.name(), "Screen");
    assert_eq!(condition.clone_box().name(), "Screen");
}

#[test]
fn impossible_screen_requirements_fail() {
    assert!(!sync_condition(Some(0), &None, &None));
    assert!(!sync_condition(
        None,
        &Some("autopilot-spec-missing-display".to_string()),
        &None
    ));
    assert!(!sync_condition(
        None,
        &None,
        &Some(vec!["autopilot-spec-missing-display".to_string()])
    ));
    assert!(!futures::executor::block_on(async_condition(
        Some(0),
        None,
        None
    )));
}
