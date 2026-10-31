use serde_json::Value;

use super::{FormError, FormField, FormFieldKind, FormSchema, FormValues};

impl FormSchema {
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
