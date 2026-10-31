use std::fmt;

use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::FormValues;

pub const AUTOPILOT_HINT: &str = "x-autopilot";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormSchema {
    pub title: String,
    pub description: Option<String>,
    #[serde(default)]
    pub groups: Vec<FormGroup>,
    pub fields: Vec<FormField>,
}

impl FormSchema {
    pub fn field(&self, key: &str) -> Option<&FormField> {
        self.fields.iter().find(|field| field.key == key)
    }

    pub fn fields_in_group<'a>(&'a self, key: &str) -> impl Iterator<Item = &'a FormField> {
        self.fields
            .iter()
            .filter(move |field| field.group.as_deref() == Some(key))
    }
}

impl fmt::Display for FormSchema {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({} fields)", self.title, self.fields.len())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormGroup {
    pub key: String,
    pub title: String,
    #[serde(default)]
    pub description: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormField {
    pub key: String,
    pub label: String,
    pub description: Option<String>,
    pub required: bool,
    #[serde(default)]
    pub group: Option<String>,
    #[serde(default)]
    pub visible_when: Option<FormVisibility>,
    pub kind: FormFieldKind,
}

impl FormField {
    /// Whether this field should be shown for `values`.
    ///
    /// Fields without a visibility rule are always shown. A rule matches only
    /// when the referenced field exists and equals the expected value.
    pub fn is_visible(&self, values: &FormValues) -> bool {
        self.visible_when
            .as_ref()
            .map(|rule| values.get(&rule.field) == Some(&rule.equals))
            .unwrap_or(true)
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormVisibility {
    pub field: String,
    pub equals: Value,
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
