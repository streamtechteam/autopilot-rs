use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};
use colored::Colorize;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

#[derive(Clone)]
pub struct LogicalCondition {
    operator: LogicalOperator,
    conditions: Vec<Box<dyn Condition>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LogicalOperator {
    And,
    Or,
    Nor,
}

impl LogicalOperator {
    fn name(&self) -> &'static str {
        match self {
            LogicalOperator::And => "AND",
            LogicalOperator::Or => "OR",
            LogicalOperator::Nor => "NOR",
        }
    }

    fn evaluate(&self, results: &[bool]) -> bool {
        match self {
            LogicalOperator::And => results.iter().all(|&x| x),
            LogicalOperator::Or => results.iter().any(|&x| x),
            LogicalOperator::Nor => !results.iter().any(|&x| x), // NOT (A OR B OR C...)
        }
    }
}

impl LogicalCondition {
    pub fn new(operator: LogicalOperator, conditions: Vec<Box<dyn Condition>>) -> Self {
        LogicalCondition { operator, conditions }
    }

    pub fn from_scheme(scheme: LogicalConditionScheme) -> Self {
        LogicalCondition {
            operator: scheme.operator,
            conditions: scheme
                .conditions
                .into_iter()
                .map(|c| c.to_condition())
                .collect::<Vec<_>>(),
        }
    }
}

impl Condition for LogicalCondition {
    fn check(&self) -> bool {
        let results: Vec<bool> = self.conditions.iter()
            .map(|condition| condition.check())
            .collect();
        
        self.operator.evaluate(&results)
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        self.operator.name()
    }

    fn create(&self) -> Result<super::ConditionScheme, AutoPilotError> {
        let mut conditions: Vec<ConditionScheme> = Vec::new();
        
        // First, ask user which logical operator they want to use
        let operators = vec!["AND", "OR", "NOR"];
        let selected_operator_idx = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose a logical operator:")
            .items(&operators)
            .default(0)
            .interact_opt()
            .map_err(|err| {
                AutoPilotError::InvalidJob(format!("Failed to select logical operator: {}", err))
            })?
            .ok_or_else(|| {
                AutoPilotError::InvalidJob("No logical operator selected".to_string())
            })?;

        let operator = match selected_operator_idx {
            0 => LogicalOperator::And,
            1 => LogicalOperator::Or,
            2 => LogicalOperator::Nor,
            _ => LogicalOperator::And, // fallback
        };

        loop {
            println!("{}", format!("[{} CONDITION]", operator.name()).yellow());
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt(format!("Do you want to add a condition to '{}'? ", operator.name()))
                .interact_opt()
                .map_err(|err| {
                    AutoPilotError::InvalidJob(format!(
                        "Failed to get condition preference: {}",
                        err
                    ))
                })?
                .unwrap_or(false)
            {
                break;
            }

            // Get available condition types
            let condition_names: Vec<String> = ConditionScheme::varient_names();

            let selected_index = Select::with_theme(&ColorfulTheme::default())
                .with_prompt("Choose a condition type:")
                .items(
                    &condition_names
                        .iter()
                        .map(|s| s.as_str())
                        .collect::<Vec<_>>(),
                )
                .default(0)
                .interact_opt()
                .map_err(|err| {
                    AutoPilotError::InvalidJob(format!("Failed to select condition type: {}", err))
                })?
                .ok_or_else(|| {
                    AutoPilotError::InvalidJob("No condition type selected".to_string())
                })?;

            let selected_condition = ConditionScheme::iter()
                .nth(selected_index)
                .expect("Error happened when creating condition")
                .to_condition();
            match selected_condition.create() {
                Ok(condition_scheme) => {
                    conditions.push(condition_scheme);
                    println!("Condition added to '{}' successfully!", operator.name());
                }
                Err(e) => {
                    eprintln!("Failed to create condition: {}", e);
                    continue;
                }
            }
        }
        
        Ok(ConditionScheme::Logical(LogicalConditionScheme {
            operator,
            conditions,
        }))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct LogicalConditionScheme {
    #[serde(rename = "operator")]
    operator: LogicalOperator,
    
    #[serde(default)]
    conditions: Vec<ConditionScheme>,
}

// Implement Serialize and Deserialize for LogicalOperator manually
impl Serialize for LogicalOperator {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(self.name())
    }
}

impl<'de> Deserialize<'de> for LogicalOperator {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        use serde::de::{Error, Visitor};
        use std::fmt;

        struct LogicalOperatorVisitor;

        impl<'de> Visitor<'de> for LogicalOperatorVisitor {
            type Value = LogicalOperator;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a string representing a logical operator (AND, OR, NOR)")
            }

            fn visit_str<E>(self, value: &str) -> Result<LogicalOperator, E>
            where
                E: Error,
            {
                match value.to_uppercase().as_str() {
                    "AND" => Ok(LogicalOperator::And),
                    "OR" => Ok(LogicalOperator::Or),
                    "NOR" => Ok(LogicalOperator::Nor),
                    _ => Err(E::custom(format!("Unknown logical operator: {}", value))),
                }
            }
        }

        deserializer.deserialize_str(LogicalOperatorVisitor)
    }
}