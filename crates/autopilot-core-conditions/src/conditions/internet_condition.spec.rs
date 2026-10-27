use super::*;

#[test]
fn from_scheme_defaults_host_and_timeout() {
    let condition = InternetCondition::from_scheme(InternetConditionScheme::default());

    assert_eq!(condition.host, "8.8.8.8");
    assert_eq!(condition.timeout, 2);
}

#[test]
fn from_scheme_uses_supplied_values() {
    let condition = InternetCondition::from_scheme(InternetConditionScheme {
        host: Some("127.0.0.1".to_string()),
        timeout: Some(5),
    });

    assert_eq!(condition.host, "127.0.0.1");
    assert_eq!(condition.timeout, 5);
    assert_eq!(condition.name(), "Internet");
    assert_eq!(condition.clone_box().name(), "Internet");
}
