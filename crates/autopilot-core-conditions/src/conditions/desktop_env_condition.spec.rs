use super::*;

#[test]
fn scheme_values_are_preserved() {
    let condition = DesktopEnvCondition::from_scheme(DesktopEnvConditionScheme {
        target: "GNOME".to_string(),
    });

    assert_eq!(condition.target, "GNOME");
    assert_eq!(condition.name(), "Desktop Environment");
    assert_eq!(condition.clone_box().name(), "Desktop Environment");
}

#[test]
fn matches_detected_environment_case_insensitively() {
    let Some(current) = get_current_de() else {
        return;
    };

    assert!(sync_condition(&current.to_lowercase()));
    assert!(sync_condition(&current.to_uppercase()));
    assert!(!sync_condition("autopilot-spec-unknown-desktop"));
    assert!(futures::executor::block_on(async_condition(
        &current.to_lowercase()
    )));
}
