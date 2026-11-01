use super::*;

#[test]
fn editor_provider_returns_supported_editors() {
    let values = values("editors").unwrap();

    assert!(values.options.iter().all(|option| !option.value.is_empty()));
    assert!(values.options.iter().all(|option| {
        autopilot_core_cross_platform::TERMINAL_EDITORS.contains(&option.value.as_str())
    }));
    assert!(!values.live);
}

#[test]
fn provider_options_carry_labels_and_values() {
    let values = values("editors").unwrap();

    assert!(!values.options.is_empty());
    assert!(
        values
            .options
            .iter()
            .all(|option| option.label == option.value)
    );
}

#[test]
fn provider_values_are_sorted_and_unique() {
    let values = values("processes").unwrap().options;
    let mut sorted = values.clone();
    sorted.sort_by(|left, right| left.value.cmp(&right.value));
    sorted.dedup_by(|left, right| left.value == right.value);

    assert_eq!(values, sorted);
}

#[test]
fn live_providers_are_marked() {
    assert!(values("processes").unwrap().live);
    assert!(values("disks").unwrap().live);
    assert!(!values("editors").unwrap().live);
}

#[test]
fn unknown_provider_is_reported() {
    let error = values("not-a-provider").unwrap_err();

    assert!(matches!(error, FormError::Unsupported(_)));
}
