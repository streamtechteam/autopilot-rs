use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::conditions::Condition;

/// Represents a custom condition that executes an arbitrary command and checks its exit code
#[derive(Clone)]
pub struct CommandCondition {
    /// The shell command to execute
    pub command: String,
    /// Whether to check for exit code 0 (true) or if output matches target (false)
    pub check_exit_code: bool,
    /// Optional target output to match (only used if check_exit_code is false)
    pub target_output: Option<String>,
    /// Per-condition polling interval override in milliseconds.
    /// `None` means the job-level `check_interval` applies.
    pub check_interval: Option<String>,
}

impl CommandCondition {
    /// Create a new custom condition that checks for exit code 0
    pub fn new(command: String) -> Self {
        CommandCondition {
            command,
            check_exit_code: true,
            target_output: None,
            check_interval: None,
        }
    }

    /// Create a new custom condition that checks for specific output
    pub fn with_output(command: String, target_output: String) -> Self {
        CommandCondition {
            command,
            check_exit_code: false,
            target_output: Some(target_output),
            check_interval: None,
        }
    }

    /// Create from a scheme (used for deserialization)
    pub fn from_scheme(scheme: CommandConditionScheme) -> Self {
        Self {
            command: scheme.command,
            check_exit_code: scheme.check_exit_code.unwrap_or(true),
            target_output: scheme.target_output,
            check_interval: scheme.check_interval,
        }
    }
}

impl Condition for CommandCondition {
    fn check(&self) -> bool {
        sync_condition(
            &self.command,
            self.check_exit_code,
            self.target_output.as_deref(),
        )
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "Command"
    }
    fn check_interval(&self) -> Option<&str> {
        self.check_interval.as_deref()
    }
}

/// Check if a custom command condition is satisfied (synchronously)
pub fn sync_condition(command: &str, check_exit_code: bool, target_output: Option<&str>) -> bool {
    match duct_sh::sh_dangerous(command).read() {
        Ok(output) => {
            if check_exit_code {
                true
            } else if let Some(target) = target_output {
                output.trim() == target
            } else {
                true
            }
        }
        Err(_) => false,
    }
}

// fallback to sync_condition for now
pub async fn async_condition(
    command: &str,
    check_exit_code: bool,
    target_output: Option<&str>,
) -> bool {
    // For now, just call sync_condition (until i decide do we even need async)
    sync_condition(command, check_exit_code, target_output)
}

/// Scheme for CommandCondition (JSON Comaptible)
#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Command condition")]
pub struct CommandConditionScheme {
    /// The shell command to execute
    #[serde(default)]
    #[schemars(
        title = "Command",
        extend(
            "x-autopilot" = {
                "required": true,
                "widget": "editor",
                "provider": "editors",
                "placeholder": "Shell command"
            }
        )
    )]
    pub command: String,
    /// Check for exit code 0 (true) or match output (false). Defaults to true.
    #[serde(default)]
    #[schemars(
        title = "Check exit code",
        description = "Leave unset to check the exit code"
    )]
    pub check_exit_code: Option<bool>,
    /// Target output to match (only used if check_exit_code is false)
    #[serde(default)]
    #[schemars(
        title = "Target output",
        extend("x-autopilot" = {"widget": "editor", "provider": "editors"})
    )]
    pub target_output: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = "Check interval",
        description = "Milliseconds between checks. Overrides the job-level check_interval."
    )]
    pub check_interval: Option<String>,
}

#[cfg(test)]
#[path = "command_condition.spec.rs"]
mod spec;
