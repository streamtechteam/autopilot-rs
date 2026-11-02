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

#[test]
fn matches_colon_separated_desktop_names_and_paths() {
    assert!(desktop_name_matches("ubuntu:GNOME", "GNOME"));
    assert!(desktop_name_matches("ubuntu:GNOME", "ubuntu"));
    assert!(desktop_name_matches("KDE;Plasma", "plasma"));
    assert!(desktop_name_matches("/usr/share/xsessions/gnome", "GNOME"));
    assert!(!desktop_name_matches("ubuntu:GNOME", "KDE"));
    assert!(!desktop_name_matches("GNOME", ""));
}
