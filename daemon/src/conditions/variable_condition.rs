use serde::{Deserialize, Serialize};
use tokio_cron_scheduler::JobScheduler;

use crate::conditions::Condition;
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
}

pub fn sync_condition(var: &str, target: &str) -> bool {
    let env_var = std::env::var(var).unwrap_or_default();
    env_var == target
}

pub async fn async_condition(var: &str, target: &str) -> bool {
    let env_var = std::env::var(var).unwrap_or_default();
    env_var == target
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct VariableConditionScheme {
    variable: String,
    target: String,
}
