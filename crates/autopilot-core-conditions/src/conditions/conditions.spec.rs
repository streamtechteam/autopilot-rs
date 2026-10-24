use super::*;
use crate::conditions::fail_condition::FailCondition;
use serde_json::json;

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

#[test]
fn every_condition_form_type_builds_a_schema() {
    let types = ConditionScheme::form_types();

    assert_eq!(types.len(), 14);
    for condition_type in types {
        let schema = ConditionScheme::form_schema(condition_type.key)
            .unwrap_or_else(|error| panic!("{}: {error}", condition_type.key));
        assert!(!schema.title.is_empty(), "{}", condition_type.key);
        assert!(!schema.fields.is_empty(), "{}", condition_type.key);
    }
}

#[test]
fn every_condition_form_round_trips() {
    let samples = [
        json!({"type":"command","condition":{"command":"echo hi","check_exit_code":false,"target_output":"hi"}}),
        json!({"type":"variable","condition":{"variable":"HOME","target":"/home/user"}}),
        json!({"type":"desktopenv","condition":{"target":"GNOME"}}),
        json!({"type":"bluetooth","condition":{"device":"Headphones","match_by_mac":false}}),
        json!({"type":"wifi","condition":{"ssid":"Home"}}),
        json!({"type":"power","condition":{"check_charging":true,"threshold":20.0,"operator":"less"}}),
        json!({"type":"resource","condition":{"resource_type":"cpu","threshold":50.0,"operator":"greater"}}),
        json!({"type":"internet","condition":{"host":"8.8.8.8","timeout":2}}),
        json!({"type":"process","condition":{"process_name":"sshd","should_be_running":true}}),
        json!({"type":"diskspace","condition":{"path":"/","min_free_gb":1.5,"max_used_gb":10.0}}),
        json!({"type":"file","condition":{"path":"/tmp","check_type":"exists","time_threshold":300,"size_threshold":1024}}),
        json!({"type":"externaldevice","condition":{"device_identifier":"usb","check_by_name":true}}),
        json!({"type":"screen","condition":{"screen_count":2,"active_screen_name":"DP-1","screen_names":["HDMI-1"]}}),
        json!({"type":"logical","condition":{"operator":"Or","conditions":[{"type":"fail","condition":null}]}}),
    ];

    for sample in samples {
        let scheme: ConditionScheme = serde_json::from_value(sample.clone()).unwrap();
        let key = scheme.form_type_key();
        let values = scheme.to_form_values().unwrap();
        let restored = ConditionScheme::from_form_values(key, values)
            .unwrap_or_else(|error| panic!("{key}: {error}"));

        assert_eq!(serde_json::to_value(restored).unwrap(), sample, "{key}");
    }
}
