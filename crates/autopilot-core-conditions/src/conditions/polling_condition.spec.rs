use super::*;
use crate::conditions::fail_condition::FailCondition;
use serde_json::json;

struct AlwaysTrue;

impl Condition for AlwaysTrue {
    fn check(&self) -> bool {
        true
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(AlwaysTrue)
    }

    fn name(&self) -> &str {
        "AlwaysTrue"
    }
}

fn wrapped(inner: Box<dyn Condition>, check_interval: Option<&str>) -> PollingCondition {
    PollingCondition::new(inner, check_interval.map(str::to_string))
}

#[test]
fn delegates_checks_to_the_inner_condition() {
    let passing = wrapped(Box::new(AlwaysTrue), Some("50"));
    let failing = wrapped(Box::new(FailCondition), Some("50"));

    assert!(passing.check());
    assert!(!failing.check());
    assert_eq!(passing.name(), "Polling");
}

#[test]
fn exposes_the_configured_interval() {
    let with_override = wrapped(Box::new(AlwaysTrue), Some("50"));
    let without_override = wrapped(Box::new(AlwaysTrue), None);

    assert_eq!(with_override.check_interval(), Some("50"));
    assert_eq!(without_override.check_interval(), None);
    assert_eq!(with_override.clone_box().check_interval(), Some("50"));
}

#[test]
fn builds_nested_conditions_from_a_scheme() {
    let scheme = PollingConditionScheme {
        condition: Box::new(ConditionScheme::Fail(FailCondition)),
        check_interval: Some("50".to_string()),
    };

    let condition = PollingCondition::from_scheme(scheme);

    assert!(!condition.check());
    assert_eq!(condition.check_interval(), Some("50"));
}

#[test]
fn deserialises_nested_polling_json() {
    let value = json!({
        "type": "polling",
        "condition": {
            "condition": {"type": "fail", "condition": null},
            "check_interval": "50",
        },
    });
    let scheme: ConditionScheme = serde_json::from_value(value).unwrap();

    let condition = scheme.to_condition();

    assert_eq!(condition.name(), "Polling");
    assert!(!condition.check());
    assert_eq!(condition.check_interval(), Some("50"));
}

#[test]
fn omits_an_unset_interval_when_serializing() {
    let scheme = ConditionScheme::Polling(PollingConditionScheme::default());

    assert_eq!(
        serde_json::to_value(scheme).unwrap(),
        json!({
            "type": "polling",
            "condition": {"condition": {"type": "fail", "condition": null}},
        })
    );
}

#[test]
fn rejects_unknown_fields() {
    let value = json!({
        "type": "polling",
        "condition": {
            "condition": {"type": "fail", "condition": null},
            "check_interval": "50",
            "unexpected": true,
        },
    });

    assert!(serde_json::from_value::<ConditionScheme>(value).is_err());
}
