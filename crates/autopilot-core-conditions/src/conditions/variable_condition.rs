use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::conditions::Condition;
#[derive(Clone)]
pub struct VariableCondition {
    variable: String,
    target: String,
    /// Per-condition polling interval override in milliseconds.
    /// `None` means the job-level `check_interval` applies.
    check_interval: Option<String>,
}
impl VariableCondition {
    pub fn new(variable: String, target: String) -> Self {
        Self {
            variable,
            target,
            check_interval: None,
        }
    }
    pub fn from_scheme(scheme: VariableConditionScheme) -> Self {
        Self {
            variable: scheme.variable,
            target: scheme.target,
            check_interval: scheme.check_interval,
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
    fn check_interval(&self) -> Option<&str> {
        self.check_interval.as_deref()
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

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Variable condition")]
pub struct VariableConditionScheme {
    #[serde(default)]
    #[schemars(
        title = "Variable",
        description = "Environment variable name without $",
        extend("x-autopilot" = {"required": true})
    )]
    variable: String,
    #[serde(default)]
    #[schemars(
        title = "Expected value",
        extend("x-autopilot" = {"required": true})
    )]
    target: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = "Check interval",
        description = "Milliseconds between checks. Overrides the job-level check_interval."
    )]
    check_interval: Option<String>,
}

#[cfg(test)]
#[path = "variable_condition.spec.rs"]
mod spec;
