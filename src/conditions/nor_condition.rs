use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};
use colored::Colorize;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;
#[derive(Clone)]
pub struct NorCondition {
    conditions: Vec<Box<dyn Condition>>,
}

impl NorCondition {
    pub fn new(conditions: Vec<Box<dyn Condition>>) -> Self {
        NorCondition { conditions }
    }

    pub fn from_scheme(scheme: NorConditionScheme) -> Self {
        NorCondition {
            conditions: scheme
                .conditions
                .into_iter()
                .map(|c| c.to_condition())
                .collect::<Vec<_>>(),
        }
    }
}

impl Condition for NorCondition {
    fn check(&self) -> bool {
        for condition in &self.conditions {
            if condition.check() {
                return false;
            }
        }
        true
    }
    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "NOR"
    }

    fn create(&self) -> Result<super::ConditionScheme, AutoPilotError> {
        let mut conditions: Vec<ConditionScheme> = Vec::new();
        loop {
            println!("{}", "[NOR CONDITION]".yellow());
            if !Confirm::with_theme(&ColorfulTheme::default())
                .with_prompt("Do you want to add a condition to 'NOR'? ")
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
                    println!("Condition added to 'AND' successfully!");
                }
                Err(e) => {
                    eprintln!("Failed to create condition: {}", e);
                    continue;
                }
            }
        }
        // self.conditions = conditions
        //     .iter()
        //     .map(|value| value.to_condition())
        //     .collect();
        Ok(ConditionScheme::Nor(NorConditionScheme {
            conditions: conditions,
        }))
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default)]
#[serde(deny_unknown_fields)]
pub struct NorConditionScheme {
    #[serde(default)]
    conditions: Vec<ConditionScheme>,
}
