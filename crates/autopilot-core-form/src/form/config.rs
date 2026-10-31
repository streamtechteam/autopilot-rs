use schemars::{JsonSchema, SchemaGenerator};
use serde::{Serialize, de::DeserializeOwned};
use serde_json::Value;

use super::{FormError, FormSchema, FormValues};

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

/// Implements [`ConditionConfig`] with all of the trait's default behavior.
///
/// Use this for configs that only need schema generation, serde round-tripping,
/// and the default (no-op) validation. Configs with cross-field validation, such
/// as `ScreenConditionScheme`, should implement the trait directly instead.
#[macro_export]
macro_rules! impl_condition_config {
    ($($type:ty),+ $(,)?) => {
        $(impl $crate::ConditionConfig for $type {})+
    };
}
