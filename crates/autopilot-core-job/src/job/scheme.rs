use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use autopilot_core_conditions::ConditionScheme;
use autopilot_core_error::AutoPilotError;
use autopilot_core_task::TaskScheme;
use autopilot_core_time::{When, to_cron_expression, validate_cron_expression};

use super::TaskMode;

/// Smallest accepted `check_interval`, in milliseconds.
pub const MIN_CHECK_INTERVAL_MS: u64 = 100;

/// Parse and validate a `check_interval` value.
pub fn parse_check_interval(value: &str) -> Result<u64, AutoPilotError> {
    let millis = value.trim().parse::<u64>().map_err(|_| {
        AutoPilotError::InvalidJob(format!(
            "check_interval must be a whole number of milliseconds, got '{value}'"
        ))
    })?;

    if millis < MIN_CHECK_INTERVAL_MS {
        return Err(AutoPilotError::InvalidJob(format!(
            "check_interval must be at least {MIN_CHECK_INTERVAL_MS} ms, got '{value}'"
        )));
    }

    Ok(millis)
}

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "AutoPilot job file")]
pub struct JobScheme {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub when: Option<When>,
    pub check_interval: Option<String>,
    pub conditions: Vec<ConditionScheme>,
    pub tasks: Vec<TaskScheme>,
    #[serde(default)]
    pub task_mode: TaskMode,
}

impl JobScheme {
    /// Validate values that the scheduler would otherwise reject at runtime.
    pub fn validate(&self) -> Result<(), AutoPilotError> {
        if let Some(interval) = self.check_interval.as_deref() {
            parse_check_interval(interval)?;
        }

        if let Some(when) = &self.when {
            match when {
                When::Cron(expression) => validate_cron_expression(expression)?,
                When::Once(value) => {
                    value.parse()?;
                }
                other => {
                    let expression = to_cron_expression(other.clone())?;
                    validate_cron_expression(&expression)?;
                }
            }
        }

        for (index, task) in self.tasks.iter().enumerate() {
            if task.timeout == Some(0) {
                return Err(AutoPilotError::InvalidJob(format!(
                    "tasks[{index}].timeout must be at least 1 second"
                )));
            }
        }

        Ok(())
    }
}

#[cfg(test)]
#[path = "scheme.spec.rs"]
mod spec;
