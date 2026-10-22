use super::*;
use crate::conditions::fail_condition::FailCondition;

#[test]
fn deserialises_condition_schemes() {
    let command: ConditionScheme = serde_json::from_str(
        r#"{"type":"command","condition":{"command":"echo hi","check_exit_code":false}}"#,
    )
    .unwrap();
    let fail: ConditionScheme =
        serde_json::from_str(r#"{"type":"fail","condition":null}"#).unwrap();
    let logical: ConditionScheme = serde_json::from_str(
        r#"{"type":"logical","condition":{"operator":"Or","conditions":[{"type":"fail","condition":null}]}}"#,
    )
    .unwrap();

    assert_eq!(command.to_condition().name(), "Command");
    assert_eq!(fail.to_condition().name(), "Fail");
    assert_eq!(logical.to_condition().name(), "Logical");
}

#[test]
fn rejects_unknown_condition_types_and_fields() {
    assert!(serde_json::from_str::<ConditionScheme>(r#"{"type":"nope","condition":{}}"#).is_err());
    assert!(
        serde_json::from_str::<ConditionScheme>(
            r#"{"type":"command","condition":{"command":"true","unexpected":true}}"#
        )
        .is_err()
    );
}

#[test]
fn variant_names_cover_every_variant() {
    let names = ConditionScheme::varient_names();

    assert_eq!(names.len(), ConditionScheme::iter().count());
    assert!(names.iter().all(|name| !name.is_empty()));
    assert!(names.contains(&"Fail".to_string()));
    assert!(names.contains(&"WiFi".to_string()));
    assert!(names.contains(&"Logical".to_string()));
}

#[test]
fn cloned_boxed_conditions_report_the_same_name() {
    let condition = ConditionScheme::Fail(FailCondition);

    assert_eq!(condition.to_condition().clone_box().name(), "Fail");
}
