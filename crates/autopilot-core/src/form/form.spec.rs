use super::*;
use serde_json::json;

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct SampleConfig {
    #[serde(default)]
    #[schemars(
        title = "WiFi network",
        extend(
            "x-autopilot" = {
                "required": true,
                "widget": "dynamic-select",
                "provider": "wifi.networks",
                "allow_custom": true
            }
        )
    )]
    ssid: String,
    #[serde(default)]
    check_exit_code: Option<bool>,
    #[serde(default)]
    retries: Option<u64>,
    #[serde(default)]
    mode: Mode,
    #[serde(default)]
    names: Vec<String>,
    #[serde(default)]
    #[schemars(extend(
        "x-autopilot" = {"required": true, "widget": "conditions", "min_items": 1}
    ))]
    conditions: Vec<NestedConfig>,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
enum Mode {
    #[default]
    And,
    Or,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
struct NestedConfig {
    value: String,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
struct UnsupportedConfig {
    nested: NestedObject,
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize, JsonSchema)]
struct NestedObject {
    value: String,
}

impl ConditionConfig for SampleConfig {}
impl ConditionConfig for UnsupportedConfig {}

fn field<'a>(schema: &'a FormSchema, key: &str) -> &'a FormField {
    schema
        .field(key)
        .unwrap_or_else(|| panic!("missing field {key}"))
}

#[test]
fn converts_required_dynamic_text_and_hint_metadata() {
    let schema = SampleConfig::form_schema();
    let ssid = field(&schema, "ssid");

    assert_eq!(schema.title, "SampleConfig");
    assert_eq!(ssid.label, "WiFi network");
    assert!(ssid.required);
    assert!(matches!(
        &ssid.kind,
        FormFieldKind::DynamicSelect {
            provider,
            multiple: false,
            allow_custom: true,
            ..
        } if provider == "wifi.networks"
    ));
}

#[test]
fn converts_optional_scalars_without_inventing_defaults() {
    let schema = SampleConfig::form_schema();

    assert!(matches!(
        &field(&schema, "check_exit_code").kind,
        FormFieldKind::Boolean { default: None }
    ));
    assert!(matches!(
        &field(&schema, "retries").kind,
        FormFieldKind::Integer {
            default: None,
            min: Some(0),
            ..
        }
    ));
}

#[test]
fn resolves_enum_references_and_defaults() {
    let schema = SampleConfig::form_schema();
    let mode = field(&schema, "mode");

    assert!(!mode.required);
    match &mode.kind {
        FormFieldKind::Select { options, default } => {
            assert_eq!(
                options,
                &[FormOption::new("And", "and"), FormOption::new("Or", "or")]
            );
            assert_eq!(default.as_deref(), Some("and"));
        }
        other => panic!("unexpected mode kind: {other:?}"),
    }
}

#[test]
fn converts_string_arrays_to_string_lists() {
    let schema = SampleConfig::form_schema();

    assert!(matches!(
        &field(&schema, "names").kind,
        FormFieldKind::StringList { default, .. } if default.is_empty()
    ));
}

#[test]
fn converts_condition_arrays_from_the_extension_hint() {
    let schema = SampleConfig::form_schema();

    assert!(matches!(
        &field(&schema, "conditions").kind,
        FormFieldKind::Conditions { min_items: 1 }
    ));
}

#[test]
fn validates_required_text_and_condition_items() {
    let schema = SampleConfig::form_schema();
    let empty = FormValues::new();
    let error = schema.validate(&empty).unwrap_err();
    assert!(matches!(error, FormError::MissingField(_)));

    let mut values = FormValues::new();
    values.insert("conditions", json!([{"value": "present"}]));
    let error = schema.validate(&values).unwrap_err();
    assert!(matches!(error, FormError::MissingField(field) if field == "ssid"));

    values.insert("ssid", json!("Home"));
    schema.validate(&values).unwrap();
}

#[test]
fn condition_config_round_trips_typed_values() {
    let config = SampleConfig {
        ssid: "Home".to_string(),
        check_exit_code: Some(false),
        retries: Some(3),
        mode: Mode::Or,
        names: vec!["one".to_string(), "two".to_string()],
        conditions: vec![NestedConfig {
            value: "nested".to_string(),
        }],
    };

    let values = config.to_form_values().unwrap();
    let restored = SampleConfig::from_form_values(values).unwrap();

    assert_eq!(restored, config);
}

#[test]
fn condition_config_rejects_missing_required_values() {
    let mut values = FormValues::new();
    values.insert("conditions", json!([{"value": "present"}]));
    let error = SampleConfig::from_form_values(values).unwrap_err();

    assert!(matches!(error, FormError::MissingField(field) if field == "ssid"));
}

#[test]
fn rejects_empty_required_condition_items() {
    let schema = SampleConfig::form_schema();
    let mut values = FormValues::new();
    values.insert("ssid", json!("Home"));
    values.insert("conditions", json!([]));
    let error = schema.validate(&values).unwrap_err();

    assert!(matches!(
        error,
        FormError::MissingField(field) if field == "conditions"
    ));
}

#[test]
fn rejects_nested_object_fields() {
    let schema = SchemaGenerator::default().into_root_schema_for::<UnsupportedConfig>();
    let error = FormSchema::from_json_schema(&schema).unwrap_err();

    assert!(matches!(error, FormError::Unsupported(_)));
}

#[test]
fn resolves_local_references_from_definitions() {
    let schema = json!({
        "type": "object",
        "title": "Referenced",
        "properties": {
            "mode": {
                "$ref": "#/definitions/Mode",
                "default": "and"
            }
        },
        "definitions": {
            "Mode": {
                "type": "string",
                "enum": ["and", "or"]
            }
        }
    });

    let form = FormSchema::from_value(&schema).unwrap();
    let mode = field(&form, "mode");

    assert!(matches!(
        &mode.kind,
        FormFieldKind::Select { default: Some(default), .. } if default == "and"
    ));
}
