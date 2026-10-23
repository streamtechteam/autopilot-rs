use autopilot_core::conditions::ConditionScheme;
use autopilot_core::cross_platform::get::get_supported_editors;
use autopilot_core::error::AutoPilotError;
use autopilot_core::form::{
    FormError, FormField, FormFieldKind, FormSchema, FormValues, providers,
};
use dialoguer::{Confirm, Editor, Input, MultiSelect, Select, theme::ColorfulTheme};
use serde_json::{Value, json};

pub fn prompt_condition() -> Result<ConditionScheme, AutoPilotError> {
    let types = ConditionScheme::form_types();
    let labels = types
        .iter()
        .map(|condition_type| condition_type.label)
        .collect::<Vec<_>>();
    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose a condition type:")
        .items(&labels)
        .default(0)
        .interact_opt()
        .map_err(dialoguer_error)?
        .ok_or_else(|| invalid("No condition type selected"))?;
    let condition_type = &types[selected];
    let schema = ConditionScheme::form_schema(condition_type.key).map_err(form_error)?;
    let values = prompt_schema(&schema)?;

    ConditionScheme::from_form_values(condition_type.key, values).map_err(form_error)
}

fn prompt_schema(schema: &FormSchema) -> Result<FormValues, AutoPilotError> {
    let mut values = FormValues::new();

    for field in &schema.fields {
        if let Some(value) = prompt_field(field)? {
            values.insert(field.key.clone(), value);
        }
    }

    schema.validate(&values).map_err(form_error)?;
    Ok(values)
}

fn prompt_field(field: &FormField) -> Result<Option<Value>, AutoPilotError> {
    if !field.required
        && !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!("Configure '{}'? (optional)", field.label))
            .default(false)
            .interact_opt()
            .map_err(dialoguer_error)?
            .unwrap_or(false)
    {
        return Ok(None);
    }

    if let Some(description) = &field.description {
        println!("{}", description);
    }

    let value = match &field.kind {
        FormFieldKind::Text {
            default,
            placeholder,
            multiline,
            editor_provider,
        } => {
            if editor_provider.as_deref() == Some("editors") && !*multiline {
                prompt_editor(field, default.as_deref())?
            } else {
                prompt_text(field, default.as_deref(), placeholder.as_deref())?
            }
        }
        FormFieldKind::Integer {
            default,
            min,
            max,
            unit,
        } => prompt_integer(field, *default, *min, *max, unit.as_deref())?,
        FormFieldKind::Number {
            default,
            min,
            max,
            unit,
        } => prompt_number(field, *default, *min, *max, unit.as_deref())?,
        FormFieldKind::Boolean { default } => Value::Bool(
            Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(&field.label)
                .default(default.unwrap_or(false))
                .interact_opt()
                .map_err(dialoguer_error)?
                .unwrap_or(false),
        ),
        FormFieldKind::Select { options, default } => {
            prompt_select(field, options, default.as_deref())?
        }
        FormFieldKind::MultiSelect { options, default } => {
            prompt_multi_select(field, options, default)?
        }
        FormFieldKind::StringList {
            default,
            placeholder,
        } => prompt_string_list(field, default, placeholder.as_deref())?,
        FormFieldKind::DynamicSelect {
            provider,
            multiple,
            allow_custom,
            default,
            placeholder,
        } => prompt_dynamic_select(
            field,
            provider,
            *multiple,
            *allow_custom,
            default,
            placeholder.as_deref(),
        )?,
        FormFieldKind::Conditions { min_items } => prompt_conditions(field, *min_items)?,
    };

    if !field.required && is_blank(field, &value) {
        return Ok(None);
    }

    Ok(Some(value))
}

fn prompt_text(
    field: &FormField,
    default: Option<&str>,
    placeholder: Option<&str>,
) -> Result<Value, AutoPilotError> {
    let mut prompt = format!("Enter {}", field.label);
    if let Some(placeholder) = placeholder {
        prompt.push_str(&format!(" ({placeholder})"));
    }

    Ok(Value::String(theme_input(
        prompt,
        !field.required,
        default.map(str::to_string),
    )?))
}

fn prompt_editor(field: &FormField, default: Option<&str>) -> Result<Value, AutoPilotError> {
    let mut editors = get_supported_editors();
    editors.insert(0, "Inline");
    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(format!("Choose an editor for {}", field.label))
        .items(&editors)
        .default(0)
        .interact_opt()
        .map_err(dialoguer_error)?
        .unwrap_or(0);

    if editors[selected] == "Inline" {
        return prompt_text(field, default, None);
    }

    let value = Editor::new()
        .executable(editors[selected])
        .edit(default.unwrap_or(""))
        .map_err(dialoguer_error)?
        .ok_or_else(|| invalid(format!("{} was not provided", field.label)))?;

    Ok(Value::String(value))
}

fn prompt_integer(
    field: &FormField,
    default: Option<i64>,
    min: Option<i64>,
    max: Option<i64>,
    unit: Option<&str>,
) -> Result<Value, AutoPilotError> {
    loop {
        let prompt = number_prompt(field, unit);
        let raw = theme_input(
            prompt,
            !field.required,
            default.map(|value| value.to_string()),
        )?;
        let raw = raw.trim();

        if raw.is_empty() {
            if !field.required {
                return Ok(Value::Null);
            }
            println!("{} is required.", field.label);
            continue;
        }

        match raw.parse::<i64>() {
            Ok(value)
                if min.is_none_or(|min| value >= min) && max.is_none_or(|max| value <= max) =>
            {
                return Ok(json!(value));
            }
            Ok(_) => println!("{} is outside the allowed range.", field.label),
            Err(_) => println!("Enter a whole number for {}.", field.label),
        }
    }
}

fn prompt_number(
    field: &FormField,
    default: Option<f64>,
    min: Option<f64>,
    max: Option<f64>,
    unit: Option<&str>,
) -> Result<Value, AutoPilotError> {
    loop {
        let prompt = number_prompt(field, unit);
        let raw = theme_input(
            prompt,
            !field.required,
            default.map(|value| value.to_string()),
        )?;
        let raw = raw.trim();

        if raw.is_empty() {
            if !field.required {
                return Ok(Value::Null);
            }
            println!("{} is required.", field.label);
            continue;
        }

        match raw.parse::<f64>() {
            Ok(value)
                if value.is_finite()
                    && min.is_none_or(|min| value >= min)
                    && max.is_none_or(|max| value <= max) =>
            {
                return Ok(json!(value));
            }
            Ok(_) => println!("{} is outside the allowed range.", field.label),
            Err(_) => println!("Enter a number for {}.", field.label),
        }
    }
}

fn prompt_select(
    field: &FormField,
    options: &[autopilot_core::form::FormOption],
    default: Option<&str>,
) -> Result<Value, AutoPilotError> {
    if options.is_empty() {
        return Err(invalid(format!("{} has no options", field.label)));
    }

    let labels = options
        .iter()
        .map(|option| option.label.as_str())
        .collect::<Vec<_>>();
    let default_index = default
        .and_then(|default| options.iter().position(|option| option.value == default))
        .unwrap_or(0);
    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(&field.label)
        .items(&labels)
        .default(default_index)
        .interact_opt()
        .map_err(dialoguer_error)?
        .ok_or_else(|| invalid(format!("No {} selected", field.label)))?;

    Ok(Value::String(options[selected].value.clone()))
}

fn prompt_multi_select(
    field: &FormField,
    options: &[autopilot_core::form::FormOption],
    default: &[String],
) -> Result<Value, AutoPilotError> {
    let labels = options
        .iter()
        .map(|option| option.label.as_str())
        .collect::<Vec<_>>();
    let defaults = options
        .iter()
        .map(|option| default.contains(&option.value))
        .collect::<Vec<_>>();
    let selected = MultiSelect::with_theme(&ColorfulTheme::default())
        .with_prompt(&field.label)
        .items(&labels)
        .defaults(&defaults)
        .interact_opt()
        .map_err(dialoguer_error)?
        .unwrap_or_default();

    Ok(json!(
        selected
            .into_iter()
            .map(|index| options[index].value.clone())
            .collect::<Vec<_>>()
    ))
}

fn prompt_string_list(
    field: &FormField,
    default: &[String],
    placeholder: Option<&str>,
) -> Result<Value, AutoPilotError> {
    let mut prompt = format!("Enter {} (comma separated)", field.label);
    if let Some(placeholder) = placeholder {
        prompt.push_str(&format!(" ({placeholder})"));
    }

    let default = (!default.is_empty()).then(|| default.join(", "));
    let raw = theme_input(prompt, !field.required, default)?;

    Ok(json!(split_values(&raw)))
}

fn prompt_dynamic_select(
    field: &FormField,
    provider: &str,
    multiple: bool,
    allow_custom: bool,
    default: &[String],
    placeholder: Option<&str>,
) -> Result<Value, AutoPilotError> {
    let options = providers::values(provider).unwrap_or_default();

    if multiple {
        let labels = options.iter().map(String::as_str).collect::<Vec<_>>();
        let defaults = options
            .iter()
            .map(|option| default.contains(option))
            .collect::<Vec<_>>();
        let mut selected = if options.is_empty() {
            Vec::new()
        } else {
            MultiSelect::with_theme(&ColorfulTheme::default())
                .with_prompt(&field.label)
                .items(&labels)
                .defaults(&defaults)
                .interact_opt()
                .map_err(dialoguer_error)?
                .unwrap_or_default()
                .into_iter()
                .map(|index| options[index].clone())
                .collect::<Vec<_>>()
        };

        if allow_custom {
            if let Some(placeholder) = placeholder {
                println!("{placeholder}");
            }
            let custom = theme_input(
                format!("Additional {} (comma separated)", field.label),
                true,
                None,
            )?;
            selected.extend(split_values(&custom));
        }

        selected.sort();
        selected.dedup();
        return Ok(json!(selected));
    }

    if options.is_empty() {
        if !allow_custom {
            return Err(invalid(format!("No values available for {}", field.label)));
        }
        return prompt_text(field, None, placeholder);
    }

    let mut labels = options.clone();
    if allow_custom {
        labels.push("<Enter custom value>".to_string());
    }
    let label_refs = labels.iter().map(String::as_str).collect::<Vec<_>>();
    let default_index = default
        .first()
        .and_then(|default| options.iter().position(|option| option == default))
        .unwrap_or(0);
    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt(&field.label)
        .items(&label_refs)
        .default(default_index)
        .interact_opt()
        .map_err(dialoguer_error)?
        .ok_or_else(|| invalid(format!("No {} selected", field.label)))?;

    if allow_custom && selected == options.len() {
        return prompt_text(field, None, placeholder);
    }

    Ok(Value::String(options[selected].clone()))
}

fn prompt_conditions(field: &FormField, min_items: usize) -> Result<Value, AutoPilotError> {
    let mut conditions = Vec::new();

    loop {
        let prompt = if conditions.is_empty() {
            format!("Add a {} condition?", field.label)
        } else {
            format!("Add another {} condition?", field.label)
        };

        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(prompt)
            .default(conditions.len() < min_items)
            .interact_opt()
            .map_err(dialoguer_error)?
            .unwrap_or(false)
        {
            if conditions.len() < min_items {
                println!("At least {min_items} condition(s) are required.");
                continue;
            }
            break;
        }

        let condition = prompt_condition()?;
        let value = serde_json::to_value(condition)
            .map_err(|error| invalid(format!("Failed to build condition: {error}")))?;
        conditions.push(value);
    }

    Ok(Value::Array(conditions))
}

fn theme_input(
    prompt: String,
    allow_empty: bool,
    default: Option<String>,
) -> Result<String, AutoPilotError> {
    let theme = ColorfulTheme::default();
    let mut input = Input::<String>::with_theme(&theme)
        .with_prompt(prompt)
        .allow_empty(allow_empty);
    if let Some(default) = default {
        input = input.default(default);
    }

    input.interact_text().map_err(dialoguer_error)
}

fn number_prompt(field: &FormField, unit: Option<&str>) -> String {
    match unit {
        Some(unit) => format!("Enter {} ({unit})", field.label),
        None => format!("Enter {}", field.label),
    }
}

fn split_values(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(str::to_string)
        .collect()
}

fn is_blank(field: &FormField, value: &Value) -> bool {
    if value.is_null() {
        return true;
    }

    match &field.kind {
        FormFieldKind::Text { .. } => value
            .as_str()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true),
        FormFieldKind::StringList { .. }
        | FormFieldKind::MultiSelect { .. }
        | FormFieldKind::Conditions { .. } => value.as_array().map(Vec::is_empty).unwrap_or(true),
        FormFieldKind::DynamicSelect { multiple, .. } if *multiple => {
            value.as_array().map(Vec::is_empty).unwrap_or(true)
        }
        FormFieldKind::DynamicSelect { .. } => value
            .as_str()
            .map(|value| value.trim().is_empty())
            .unwrap_or(true),
        _ => false,
    }
}

fn invalid(message: impl Into<String>) -> AutoPilotError {
    AutoPilotError::InvalidJob(message.into())
}

fn form_error(error: FormError) -> AutoPilotError {
    invalid(error.to_string())
}

fn dialoguer_error(error: dialoguer::Error) -> AutoPilotError {
    invalid(error.to_string())
}
