use std::fmt;

use serde::{Deserialize, Serialize};

pub const AUTOPILOT_HINT: &str = "x-autopilot";

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FormSchema {
    pub title: String,
    pub description: Option<String>,
    pub fields: Vec<FormField>,
}

impl FormSchema {
    pub fn field(&self, key: &str) -> Option<&FormField> {
        self.fields.iter().find(|field| field.key == key)
    }
}

impl fmt::Display for FormSchema {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{} ({} fields)", self.title, self.fields.len())
    }
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
