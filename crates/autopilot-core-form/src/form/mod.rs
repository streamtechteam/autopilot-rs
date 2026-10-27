use std::fmt;

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use serde_json::{Map, Value};
use thiserror::Error;

pub mod providers;

pub const AUTOPILOT_HINT: &str = "x-autopilot";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormSchema {
    pub title: String,
    pub description: Option<String>,
    pub fields: Vec<FormField>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormField {
    pub key: String,
    pub label: String,
    pub description: Option<String>,
    pub required: bool,
    pub kind: FormFieldKind,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum FormFieldKind {
    Text {
        default: Option<String>,
        placeholder: Option<String>,
        multiline: bool,
        editor_provider: Option<String>,
    },
    Integer {
        default: Option<i64>,
        min: Option<i64>,
        max: Option<i64>,
        unit: Option<String>,
    },
    Number {
        default: Option<f64>,
        min: Option<f64>,
        max: Option<f64>,
        unit: Option<String>,
    },
    Boolean {
        default: Option<bool>,
    },
    Select {
        options: Vec<FormOption>,
        default: Option<String>,
    },
    MultiSelect {
        options: Vec<FormOption>,
        default: Vec<String>,
    },
    StringList {
        default: Vec<String>,
        placeholder: Option<String>,
    },
    DynamicSelect {
        provider: String,
        multiple: bool,
        allow_custom: bool,
        default: Vec<String>,
        placeholder: Option<String>,
    },
    Conditions {
        min_items: usize,
    },
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormOption {
    pub label: String,
    pub value: String,
}

impl FormOption {
    pub fn new(label: impl Into<String>, value: impl Into<String>) -> Self {
        Self {
            label: label.into(),
            value: value.into(),
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FormValues(Map<String, Value>);

impl FormValues {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn from_map(map: Map<String, Value>) -> Self {
        Self(map)
    }

    pub fn into_map(self) -> Map<String, Value> {
        self.0
    }

    pub fn as_map(&self) -> &Map<String, Value> {
        &self.0
    }

    pub fn insert(&mut self, key: impl Into<String>, value: Value) -> Option<Value> {
        self.0.insert(key.into(), value)
    }

    pub fn get(&self, key: &str) -> Option<&Value> {
        self.0.get(key)
    }

    pub fn remove(&mut self, key: &str) -> Option<Value> {
        self.0.remove(key)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn into_value(self) -> Value {
        Value::Object(self.0)
    }
}

#[derive(Debug, Error)]
pub enum FormError {
    #[error("invalid form schema: {0}")]
    InvalidSchema(String),
    #[error("invalid value for '{field}': {message}")]
    InvalidValue { field: String, message: String },
    #[error("missing required field '{0}'")]
    MissingField(String),
    #[error("unsupported schema: {0}")]
    Unsupported(String),
    #[error("form provider error: {0}")]
    Provider(String),
    #[error("form serialization error: {0}")]
    Serialization(String),
}

impl FormSchema {
    pub fn from_json_schema(schema: &Schema) -> Result<Self, FormError> {
        Self::from_value(schema.as_value())
    }

    pub fn from_value(root: &Value) -> Result<Self, FormError> {
        let root = resolve_ref(root, root)?;
        let object = root
            .as_object()
            .ok_or_else(|| FormError::InvalidSchema("root schema must be an object".to_string()))?;

        let fields = object
            .get("properties")
            .and_then(Value::as_object)
            .ok_or_else(|| FormError::InvalidSchema("root schema has no properties".to_string()))?;

        let required = object
            .get("required")
            .and_then(Value::as_array)
            .map(|values| {
                values
                    .iter()
                    .filter_map(Value::as_str)
                    .map(str::to_string)
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();

        let mut parsed = Vec::with_capacity(fields.len());
        for (key, field_schema) in fields {
            parsed.push(parse_field(root, key, field_schema, &required)?);
        }

        Ok(Self {
            title: object
                .get("title")
                .and_then(Value::as_str)
                .unwrap_or("Form")
                .to_string(),
            description: object
                .get("description")
                .and_then(Value::as_str)
                .map(str::to_string),
            fields: parsed,
        })
    }

    pub fn validate(&self, values: &FormValues) -> Result<(), FormError> {
        for field in &self.fields {
            let value = values.get(&field.key);

            if field.required && value_is_empty(field, value) {
                return Err(FormError::MissingField(field.key.clone()));
            }

            if let FormFieldKind::Conditions { min_items } = field.kind {
                let count = value
                    .and_then(Value::as_array)
                    .map(|items| items.len())
                    .unwrap_or(0);
                if (field.required || count > 0) && count < min_items {
                    return Err(FormError::InvalidValue {
                        field: field.key.clone(),
                        message: format!("at least {min_items} item(s) required"),
                    });
                }
            }
        }

        Ok(())
    }

    pub fn field(&self, key: &str) -> Option<&FormField> {
        self.fields.iter().find(|field| field.key == key)
    }
}

fn value_is_empty(field: &FormField, value: Option<&Value>) -> bool {
    let Some(value) = value else {
        return true;
    };

    match &field.kind {
        FormFieldKind::Text { .. } => value
            .as_str()
            .map(|text| text.trim().is_empty())
            .unwrap_or(true),
        FormFieldKind::StringList { .. } | FormFieldKind::Conditions { .. } => {
            value.as_array().map(Vec::is_empty).unwrap_or(true)
        }
        FormFieldKind::DynamicSelect { multiple: true, .. } | FormFieldKind::MultiSelect { .. } => {
            value.as_array().map(Vec::is_empty).unwrap_or(true)
        }
        FormFieldKind::DynamicSelect {
            multiple: false, ..
        } => value
            .as_str()
            .map(|text| text.trim().is_empty())
            .unwrap_or(true),
        _ => false,
    }
}

fn parse_field(
    root: &Value,
    key: &str,
    raw_schema: &Value,
    required: &[String],
) -> Result<FormField, FormError> {
    let schema = field_schema(root, raw_schema)?;
    let hint = FormHint::from_schema(&schema)?;
    let schema_object = schema
        .as_object()
        .ok_or_else(|| FormError::InvalidSchema(format!("field '{key}' is not an object")))?;

    let required = required.iter().any(|name| name == key) || hint.required.unwrap_or(false);
    let label = schema_object
        .get("title")
        .and_then(Value::as_str)
        .map(str::to_string)
        .unwrap_or_else(|| humanize(key));
    let description = schema_object
        .get("description")
        .and_then(Value::as_str)
        .map(str::to_string);

    let kind = if hint.widget.as_deref() == Some("conditions") {
        FormFieldKind::Conditions {
            min_items: hint.min_items.unwrap_or(1),
        }
    } else if hint.widget.as_deref() == Some("dynamic-select") {
        parse_dynamic_select(&schema, &hint, key)?
    } else if hint.widget.as_deref() == Some("select") {
        let options = match hint.options.clone() {
            Some(options) => options,
            None => enum_options(&schema).transpose()?.unwrap_or_default(),
        };
        FormFieldKind::Select {
            options,
            default: default_string(&schema),
        }
    } else {
        parse_kind(root, &schema, &hint, key)?
    };

    Ok(FormField {
        key: key.to_string(),
        label,
        description,
        required,
        kind,
    })
}

fn parse_kind(
    root: &Value,
    schema: &Value,
    hint: &FormHint,
    key: &str,
) -> Result<FormFieldKind, FormError> {
    if let Some(options) = enum_options(schema) {
        return Ok(FormFieldKind::Select {
            options: options?,
            default: default_string(schema),
        });
    }

    let schema_type = schema_type(schema)?;
    match schema_type.as_deref() {
        Some("string") => Ok(FormFieldKind::Text {
            default: default_string(schema),
            placeholder: hint.placeholder.clone(),
            multiline: hint.multiline.unwrap_or(false),
            editor_provider: hint.provider.clone(),
        }),
        Some("integer") => Ok(FormFieldKind::Integer {
            default: default_i64(schema),
            min: limit_i64(schema, "minimum"),
            max: limit_i64(schema, "maximum"),
            unit: hint.unit.clone(),
        }),
        Some("number") => Ok(FormFieldKind::Number {
            default: default_f64(schema),
            min: limit_f64(schema, "minimum"),
            max: limit_f64(schema, "maximum"),
            unit: hint.unit.clone(),
        }),
        Some("boolean") => Ok(FormFieldKind::Boolean {
            default: default_bool(schema),
        }),
        Some("array") => parse_array(root, schema, hint, key),
        Some(other) => Err(FormError::Unsupported(format!(
            "field '{key}' has unsupported type '{other}'"
        ))),
        None => Err(FormError::Unsupported(format!(
            "field '{key}' has no usable type"
        ))),
    }
}

fn parse_array(
    root: &Value,
    schema: &Value,
    hint: &FormHint,
    key: &str,
) -> Result<FormFieldKind, FormError> {
    let items = schema.get("items").ok_or_else(|| {
        FormError::Unsupported(format!("array field '{key}' has no items schema"))
    })?;
    let items = resolve_ref(items, root).unwrap_or(items);

    if let Some(options) = enum_options(items) {
        return Ok(FormFieldKind::MultiSelect {
            options: options?,
            default: default_strings(schema),
        });
    }

    match schema_type(items)?.as_deref() {
        Some("string") => Ok(FormFieldKind::StringList {
            default: default_strings(schema),
            placeholder: hint.placeholder.clone(),
        }),
        Some(other) => Err(FormError::Unsupported(format!(
            "array field '{key}' has unsupported item type '{other}'"
        ))),
        None => Err(FormError::Unsupported(format!(
            "array field '{key}' has no item type"
        ))),
    }
}

fn parse_dynamic_select(
    schema: &Value,
    hint: &FormHint,
    key: &str,
) -> Result<FormFieldKind, FormError> {
    let provider = hint.provider.clone().ok_or_else(|| {
        FormError::InvalidSchema(format!("dynamic field '{key}' has no provider"))
    })?;

    Ok(FormFieldKind::DynamicSelect {
        provider,
        multiple: hint.multiple.unwrap_or(false),
        allow_custom: hint.allow_custom.unwrap_or(true),
        default: default_strings(schema),
        placeholder: hint.placeholder.clone(),
    })
}

#[derive(Clone, Debug, Default, Deserialize)]
struct FormHint {
    #[serde(default)]
    required: Option<bool>,
    #[serde(default)]
    widget: Option<String>,
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    multiple: Option<bool>,
    #[serde(default)]
    allow_custom: Option<bool>,
    #[serde(default)]
    min_items: Option<usize>,
    #[serde(default)]
    placeholder: Option<String>,
    #[serde(default)]
    unit: Option<String>,
    #[serde(default)]
    options: Option<Vec<FormOption>>,
    #[serde(default)]
    multiline: Option<bool>,
}

impl FormHint {
    fn from_schema(schema: &Value) -> Result<Self, FormError> {
        let Some(value) = schema.get(AUTOPILOT_HINT) else {
            return Ok(Self::default());
        };

        serde_json::from_value(value.clone())
            .map_err(|error| FormError::InvalidSchema(format!("invalid {AUTOPILOT_HINT}: {error}")))
    }
}

fn field_schema(root: &Value, schema: &Value) -> Result<Value, FormError> {
    let Some(reference) = schema.get("$ref").and_then(Value::as_str) else {
        return Ok(schema.clone());
    };

    let resolved = resolve_pointer(root, reference)?;
    let mut merged = resolved.as_object().cloned().ok_or_else(|| {
        FormError::InvalidSchema(format!("reference '{reference}' is not an object"))
    })?;

    if let Some(overrides) = schema.as_object() {
        for (key, value) in overrides {
            if key != "$ref" {
                merged.insert(key.clone(), value.clone());
            }
        }
    }

    Ok(Value::Object(merged))
}

fn resolve_ref<'a>(value: &'a Value, root: &'a Value) -> Result<&'a Value, FormError> {
    if let Some(reference) = value.get("$ref").and_then(Value::as_str) {
        resolve_pointer(root, reference)
    } else {
        Ok(value)
    }
}

fn resolve_pointer<'a>(root: &'a Value, reference: &str) -> Result<&'a Value, FormError> {
    let pointer = reference.strip_prefix('#').ok_or_else(|| {
        FormError::InvalidSchema(format!(
            "only local references are supported: '{reference}'"
        ))
    })?;

    root.pointer(pointer)
        .ok_or_else(|| FormError::InvalidSchema(format!("unresolved reference '{reference}'")))
}

fn enum_options(schema: &Value) -> Option<Result<Vec<FormOption>, FormError>> {
    schema.get("enum").and_then(Value::as_array).map(|values| {
        values
            .iter()
            .map(|value| {
                value
                    .as_str()
                    .map(|value| FormOption::new(humanize(value), value))
                    .ok_or_else(|| {
                        FormError::Unsupported("only string enums are supported".to_string())
                    })
            })
            .collect()
    })
}

fn schema_type(schema: &Value) -> Result<Option<String>, FormError> {
    let Some(value) = schema.get("type") else {
        return Ok(None);
    };

    let selected = match value {
        Value::String(value) => Some(value.as_str()),
        Value::Array(values) => values
            .iter()
            .filter_map(Value::as_str)
            .find(|value| *value != "null"),
        _ => {
            return Err(FormError::InvalidSchema(
                "'type' must be a string or array".to_string(),
            ));
        }
    };

    Ok(selected.map(str::to_string))
}

fn default_value(schema: &Value) -> Option<&Value> {
    schema.get("default").filter(|value| !value.is_null())
}

fn default_string(schema: &Value) -> Option<String> {
    default_value(schema)
        .and_then(Value::as_str)
        .map(str::to_string)
}

fn default_bool(schema: &Value) -> Option<bool> {
    default_value(schema).and_then(Value::as_bool)
}

fn default_i64(schema: &Value) -> Option<i64> {
    default_value(schema).and_then(|value| {
        value
            .as_i64()
            .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
    })
}

fn default_f64(schema: &Value) -> Option<f64> {
    default_value(schema).and_then(Value::as_f64)
}

fn default_strings(schema: &Value) -> Vec<String> {
    schema
        .get("default")
        .and_then(Value::as_array)
        .map(|values| {
            values
                .iter()
                .filter_map(Value::as_str)
                .map(str::to_string)
                .collect()
        })
        .unwrap_or_default()
}

fn limit_i64(schema: &Value, key: &str) -> Option<i64> {
    schema.get(key).and_then(|value| {
        value
            .as_i64()
            .or_else(|| value.as_u64().and_then(|value| i64::try_from(value).ok()))
    })
}

fn limit_f64(schema: &Value, key: &str) -> Option<f64> {
    schema.get(key).and_then(Value::as_f64)
}

fn humanize(value: &str) -> String {
    let spaced = value.replace('_', " ");
    let mut chars = spaced.chars();
    match chars.next() {
        Some(first) => format!("{}{}", first.to_uppercase(), chars.as_str()),
        None => String::new(),
    }
}

pub trait ConditionConfig: Clone + Serialize + DeserializeOwned + JsonSchema {
    fn try_form_schema() -> Result<FormSchema, FormError> {
        let schema = SchemaGenerator::default().into_root_schema_for::<Self>();
        FormSchema::from_json_schema(&schema)
    }

    fn form_schema() -> FormSchema {
        Self::try_form_schema().expect("condition schema should produce a valid form")
    }

    fn to_form_values(&self) -> Result<FormValues, FormError> {
        match serde_json::to_value(self)
            .map_err(|error| FormError::Serialization(error.to_string()))?
        {
            Value::Object(values) => Ok(FormValues::from_map(values)),
            _ => Err(FormError::Serialization(
                "condition config must serialize to an object".to_string(),
            )),
        }
    }

    fn from_form_values(values: FormValues) -> Result<Self, FormError> {
        Self::form_schema().validate(&values)?;
        let config: Self = serde_json::from_value(values.into_value())
            .map_err(|error| FormError::Serialization(error.to_string()))?;
        config.validate()?;
        Ok(config)
    }

    fn validate(&self) -> Result<(), FormError> {
        Ok(())
    }
}

impl fmt::Display for FormSchema {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({} fields)", self.title, self.fields.len())
    }
}

#[cfg(test)]
#[path = "form.spec.rs"]
mod spec;

//  TODO
// need to work more on this, i think its not ready for GUI update yet
