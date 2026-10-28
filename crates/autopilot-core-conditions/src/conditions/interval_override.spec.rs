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

fn fail_entry(check_interval: Option<&str>) -> ConditionEntry {
    ConditionEntry {
        scheme: ConditionScheme::Fail(FailCondition),
        check_interval: check_interval.map(str::to_string),
    }
}

#[test]
fn deserialises_an_entry_with_an_override() {
    let value = json!({
        "type": "fail",
        "check_interval": "50",
        "condition": null,
    });
    let entry: ConditionEntry = serde_json::from_value(value).unwrap();

    assert_eq!(entry.check_interval.as_deref(), Some("50"));
    assert_eq!(entry.scheme.to_condition().name(), "Fail");
}

#[test]
fn missing_or_null_intervals_mean_no_override() {
    let missing = json!({"type": "fail", "condition": null});
    let missing: ConditionEntry = serde_json::from_value(missing).unwrap();
    let null = json!({"type": "fail", "check_interval": null, "condition": null});
    let null: ConditionEntry = serde_json::from_value(null).unwrap();

    assert_eq!(missing.check_interval, None);
    assert!(null.check_interval.is_none());
}

#[test]
fn rejects_non_string_intervals() {
    let value = json!({"type": "fail", "check_interval": 50, "condition": null});

    assert!(serde_json::from_value::<ConditionEntry>(value).is_err());
}

#[test]
fn rejects_unknown_fields_inside_the_condition() {
    let value = json!({
        "type": "command",
        "check_interval": "50",
        "condition": {"command": "true", "unexpected": true},
    });

    assert!(serde_json::from_value::<ConditionEntry>(value).is_err());
}

#[test]
fn serialises_the_override_next_to_the_condition() {
    let with_override = fail_entry(Some("50"));
    let without_override = fail_entry(None);

    assert_eq!(
        serde_json::to_value(with_override).unwrap(),
        json!({
            "type": "fail",
            "condition": null,
            "check_interval": "50",
        })
    );
    assert_eq!(
        serde_json::to_value(without_override).unwrap(),
        json!({"type": "fail", "condition": null})
    );
}

#[test]
fn into_condition_exposes_the_override() {
    let wrapped = fail_entry(Some("50")).into_condition();
    let bare = fail_entry(None).into_condition();

    assert_eq!(wrapped.check_interval(), Some("50"));
    assert_eq!(wrapped.name(), "Fail");
    assert!(!wrapped.check());
    assert_eq!(bare.check_interval(), None);
    assert!(wrapped.clone_box().check_interval().is_some());
}

#[test]
fn decorator_delegates_to_the_inner_condition() {
    let condition = IntervalOverride::new(Box::new(AlwaysTrue), Some("50".to_string()));

    assert!(condition.check());
    assert_eq!(condition.name(), "AlwaysTrue");
    assert_eq!(condition.check_interval(), Some("50"));
}

#[test]
fn schema_documents_the_override() {
    let schema = schemars::schema_for!(ConditionEntry);

    assert!(serde_json::to_string(&schema).unwrap().contains("check_interval"));
}
