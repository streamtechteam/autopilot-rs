use std::io::ErrorKind;

use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use duct::cmd;
use log::error;
use serde::{Deserialize, Serialize};

use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};

/// Represents a custom condition that executes an arbitrary command and checks its exit code
#[derive(Clone)]
pub struct CommandCondition {
    /// The shell command to execute
    pub command: String,
    /// Whether to check for exit code 0 (true) or if output matches target (false)
    pub check_exit_code: bool,
    /// Optional target output to match (only used if check_exit_code is false)
    pub target_output: Option<String>,
}

impl CommandCondition {
    /// Create a new custom condition that checks for exit code 0
    pub fn new(command: String) -> Self {
        CommandCondition {
            command,
            check_exit_code: true,
            target_output: None,
        }
    }

    /// Create a new custom condition that checks for specific output
    pub fn with_output(command: String, target_output: String) -> Self {
        CommandCondition {
            command,
            check_exit_code: false,
            target_output: Some(target_output),
        }
    }

    /// Create from a scheme (used for deserialization)
    pub fn from_scheme(scheme: CommandConditionScheme) -> Self {
        Self {
            command: scheme.command,
            check_exit_code: scheme.check_exit_code.unwrap_or(true),
            target_output: scheme.target_output,
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

    fn create(&self) -> Result<ConditionScheme, AutoPilotError> {
        let command = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter command to execute:")
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let check_exit_code = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Check command exit code (success/failure)?")
            .interact_opt()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?
            .unwrap_or(true);

        let target_output = if !check_exit_code {
            let output = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter expected output to match (optional, if you check exit code):")
                .interact_text()
                .map_err(|err| AutoPilotError::Condition(err.to_string()))?;
            Some(output)
        } else {
            None
        };

        Ok(ConditionScheme::Command(CommandConditionScheme {
            command,
            check_exit_code: Some(check_exit_code),
            target_output,
        }))
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
#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct CommandConditionScheme {
    /// The shell command to execute
    #[serde(default)]
    pub command: String,
    /// Check for exit code 0 (true) or match output (false). Defaults to true.
    #[serde(default)]
    pub check_exit_code: Option<bool>,
    /// Target output to match (only used if check_exit_code is false)
    #[serde(default)]
    pub target_output: Option<String>,
}
