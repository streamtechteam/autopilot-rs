use super::*;

#[test]
fn editor_provider_returns_supported_editors() {
    let values = values("editors").unwrap();

    assert!(values.iter().all(|value| !value.is_empty()));
    assert!(
        values
            .iter()
            .all(|value| crate::cross_platform::TERMINAL_EDITORS.contains(&value.as_str()))
    );
}

#[test]
fn provider_values_are_sorted_and_unique() {
    let values = values("processes").unwrap();
    let mut sorted = values.clone();
    sorted.sort();
    sorted.dedup();

    assert_eq!(values, sorted);
}

#[test]
fn unknown_provider_is_reported() {
    let error = values("not-a-provider").unwrap_err();

    assert!(matches!(error, FormError::Unsupported(_)));
}
