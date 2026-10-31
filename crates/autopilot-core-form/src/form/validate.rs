use serde_json::Value;

use super::{FormError, FormField, FormFieldError, FormFieldKind, FormSchema, FormValues};

impl FormSchema {
    pub fn validate(&self, values: &FormValues) -> Result<(), FormError> {
        self.fields
            .iter()
            .try_for_each(|field| field.validate(values))
    }

    /// Validates a single field by key, for inline/live validation in a UI.
    pub fn validate_field(&self, key: &str, values: &FormValues) -> Result<(), FormError> {
        let field = self
            .field(key)
            .ok_or_else(|| FormError::InvalidSchema(format!("unknown form field '{key}'")))?;

        field.validate(values)
    }

    /// Returns every invalid field, in field order, for inline error display.
    pub fn errors(&self, values: &FormValues) -> Vec<FormFieldError> {
        self.fields
            .iter()
            .filter_map(|field| {
                field.validate(values).err().map(|error| FormFieldError {
                    field: field.key.clone(),
                    message: error.to_string(),
                })
            })
            .collect()
    }
}

impl FormField {
    pub fn validate(&self, values: &FormValues) -> Result<(), FormError> {
        let value = values.get(&self.key);

        if self.required && value_is_empty(self, value) {
            return Err(FormError::MissingField(self.key.clone()));
        }

        if let FormFieldKind::Conditions { min_items } = self.kind {
            let count = value
                .and_then(Value::as_array)
                .map(|items| items.len())
                .unwrap_or(0);
            if (self.required || count > 0) && count < min_items {
                return Err(FormError::InvalidValue {
                    field: self.key.clone(),
                    message: format!("at least {min_items} item(s) required"),
                });
            }
        }

        Ok(())
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
