use std::borrow::Cow;

use schemars::{JsonSchema, Schema, SchemaGenerator};
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::conditions::{Condition, ConditionScheme};

/// One entry of a job's `conditions` list: the condition plus its optional
/// polling-interval override.
///
/// In job files the override sits next to the condition's `type`:
///
/// ```jsonc
/// { "type": "command", "check_interval": "50", "condition": { ... } }
/// ```
#[derive(Clone, Debug)]
pub struct ConditionEntry {
    pub scheme: ConditionScheme,
    pub check_interval: Option<String>,
}

impl ConditionEntry {
    pub fn into_condition(self) -> Box<dyn Condition> {
        let condition = self.scheme.to_condition();
        match self.check_interval {
            Some(check_interval) => {
                let wrapped: Box<dyn Condition> =
                    Box::new(IntervalOverride::new(condition, Some(check_interval)));
                wrapped
            }
            None => condition,
        }
    }
}

impl<'de> Deserialize<'de> for ConditionEntry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut fields = serde_json::Map::deserialize(deserializer)?;
        let check_interval = match fields.remove("check_interval") {
            None | Some(serde_json::Value::Null) => None,
            Some(raw) => {
                let parsed = serde_json::from_value(raw);
                Some(parsed.map_err(serde::de::Error::custom)?)
            }
        };
        let rest = serde_json::Value::Object(fields);
        let scheme = ConditionScheme::deserialize(&rest).map_err(serde::de::Error::custom)?;
        Ok(Self {
            scheme,
            check_interval,
        })
    }
}

impl Serialize for ConditionEntry {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let value = serde_json::to_value(&self.scheme);
        let mut value = value.map_err(serde::ser::Error::custom)?;
        let fields = value
            .as_object_mut()
            .ok_or_else(|| serde::ser::Error::custom("condition scheme must be an object"))?;
        if let Some(check_interval) = &self.check_interval {
            fields.insert(
                "check_interval".to_string(),
                serde_json::Value::String(check_interval.clone()),
            );
        }
        fields.serialize(serializer)
    }
}

impl JsonSchema for ConditionEntry {
    fn schema_name() -> Cow<'static, str> {
        Cow::Borrowed("Condition entry")
    }

    fn json_schema(generator: &mut SchemaGenerator) -> Schema {
        let mut value = serde_json::to_value(generator.subschema_for::<ConditionScheme>())
            .unwrap_or(serde_json::Value::Null);
        let variants = value
            .get_mut("anyOf")
            .and_then(serde_json::Value::as_array_mut);
        if let Some(variants) = variants {
            for variant in variants {
                let properties = variant
                    .get_mut("properties")
                    .and_then(serde_json::Value::as_object_mut);
                if let Some(properties) = properties {
                    properties.insert(
                        "check_interval".to_owned(),
                        serde_json::to_value(generator.subschema_for::<Option<String>>())
                            .unwrap_or(serde_json::Value::Null),
                    );
                }
            }
        }
        serde_json::from_value(value)
            .unwrap_or_else(|_| generator.subschema_for::<ConditionScheme>())
    }
}

/// A condition wrapped with a polling-interval override.
#[derive(Clone)]
pub struct IntervalOverride {
    inner: Box<dyn Condition>,
    check_interval: Option<String>,
}

impl IntervalOverride {
    pub fn new(inner: Box<dyn Condition>, check_interval: Option<String>) -> Self {
        Self {
            inner,
            check_interval,
        }
    }
}

impl Condition for IntervalOverride {
    fn check(&self) -> bool {
        self.inner.check()
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        self.inner.name()
    }

    fn check_interval(&self) -> Option<&str> {
        self.check_interval
            .as_deref()
            .or_else(|| self.inner.check_interval())
    }
}

#[cfg(test)]
#[path = "interval_override.spec.rs"]
mod spec;
