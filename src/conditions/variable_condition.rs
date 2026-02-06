use serde::{Deserialize, Serialize};

use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};
use dialoguer::{Input, theme::ColorfulTheme};
#[derive(Clone)]
pub struct VariableCondition {
    variable: String,
    target: String,
}
impl VariableCondition {
    pub fn new(variable: String, target: String) -> Self {
        Self { variable, target }
    }
    pub fn from_scheme(scheme: VariableConditionScheme) -> Self {
        Self {
            variable: scheme.variable,
            target: scheme.target,
        }
    }
}
impl Condition for VariableCondition {
    fn check(&self) -> bool {
        sync_condition(&self.variable, &self.target)
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "Variable"
    }

    fn create(&self) -> Result<ConditionScheme, AutoPilotError> {
        let variable = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter environment variable name:")
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let target = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter expected value:")
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        Ok(ConditionScheme::Variable(VariableConditionScheme {
            variable,
            target,
        }))
    }
}

pub fn sync_condition(var: &str, target: &str) -> bool {
    let env_var = std::env::var(var).unwrap_or_default();
    env_var == target
}

pub async fn async_condition(var: &str, target: &str) -> bool {
    let env_var = std::env::var(var).unwrap_or_default();
    env_var == target
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct VariableConditionScheme {
    #[serde(default)]
    variable: String,
    #[serde(default)]
    target: String,
}
