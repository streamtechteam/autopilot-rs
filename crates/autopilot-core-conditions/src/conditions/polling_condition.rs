use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::conditions::{Condition, ConditionScheme};

#[derive(Clone)]
pub struct PollingCondition {
    inner: Box<dyn Condition>,
    check_interval: Option<String>,
}

impl PollingCondition {
    pub fn new(inner: Box<dyn Condition>, check_interval: Option<String>) -> Self {
        Self {
            inner,
            check_interval,
        }
    }

    pub fn from_scheme(scheme: PollingConditionScheme) -> Self {
        Self {
            inner: scheme.condition.to_condition(),
            check_interval: scheme.check_interval,
        }
    }
}

impl Condition for PollingCondition {
    fn check(&self) -> bool {
        self.inner.check()
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "Polling"
    }

    fn check_interval(&self) -> Option<&str> {
        self.check_interval.as_deref()
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Polling condition")]
pub struct PollingConditionScheme {
    #[serde(default)]
    #[schemars(
        title = "Condition",
        extend("x-autopilot" = {"required": true})
    )]
    condition: Box<ConditionScheme>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = "Check interval",
        description = "Milliseconds between checks. Overrides the job-level check_interval."
    )]
    check_interval: Option<String>,
}

#[cfg(test)]
#[path = "polling_condition.spec.rs"]
mod spec;
