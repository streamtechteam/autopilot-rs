use duct::cmd;
use serde::{Deserialize, Serialize};
use log::error;

use crate::conditions::Condition;

/// Represents a custom condition that executes an arbitrary command and checks its exit code
#[derive(Clone)]
pub struct CustomCondition {
    /// The shell command to execute
    pub command: String,
    /// Whether to check for exit code 0 (true) or if output matches target (false)
    pub check_exit_code: bool,
    /// Optional target output to match (only used if check_exit_code is false)
    pub target_output: Option<String>,
}

impl CustomCondition {
    /// Create a new custom condition that checks for exit code 0
    pub fn new(command: String) -> Self {
        CustomCondition {
            command,
            check_exit_code: true,
            target_output: None,
        }
    }

    /// Create a new custom condition that checks for specific output
    pub fn with_output(command: String, target_output: String) -> Self {
        CustomCondition {
            command,
            check_exit_code: false,
            target_output: Some(target_output),
        }
    }

    /// Create from a scheme (used for deserialization)
    pub fn from_scheme(scheme: CustomConditionScheme) -> Self {
        Self {
            command: scheme.command,
            check_exit_code: scheme.check_exit_code.unwrap_or(true),
            target_output: scheme.target_output,
        }
    }
}

impl Condition for CustomCondition {
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
}

/// Check if a custom command condition is satisfied (synchronously)
pub fn sync_condition(command: &str, check_exit_code: bool, target_output: Option<&str>) -> bool {
    let shell: &str;
    let args: Vec<&str>;

    #[cfg(target_os = "windows")]
    {
        shell = "powershell";
        args = vec!["-NoProfile", "-Command", command];
    }

    #[cfg(target_os = "linux")]
    {
        shell = "sh";
        args = vec!["-c", command];
    }

    #[cfg(target_os = "macos")]
    {
        shell = "zsh";
        args = vec!["-c", command];
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        shell = "sh";
        args = vec!["-c", command];
    }

    match cmd(shell, args).read() {
        Ok(output) => {
            if check_exit_code {
                // Just check if the command succeeded (exit code 0)
                true
            } else if let Some(target) = target_output {
                // Check if output matches target
                output.trim() == target
            } else {
                // No target specified, just check success
                true
            }
        }
        Err(e) => {
            error!("Error executing custom condition command '{}': {}", command, e);
            false
        }
    }
}

/// Check if a custom command condition is satisfied (asynchronously)
pub async fn async_condition(
    command: &str,
    check_exit_code: bool,
    target_output: Option<&str>,
) -> bool {
    // For now, just call sync_condition since command execution is typically fast
    // In a real async implementation, you'd use tokio::process::Command
    sync_condition(command, check_exit_code, target_output)
}

/// Scheme for deserializing CustomCondition from JSON/JSONC
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct CustomConditionScheme {
    /// The shell command to execute
    pub command: String,
    /// Check for exit code 0 (true) or match output (false). Defaults to true.
    #[serde(default)]
    pub check_exit_code: Option<bool>,
    /// Target output to match (only used if check_exit_code is false)
    #[serde(default)]
    pub target_output: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_custom_condition_creation() {
        let condition = CustomCondition::new("true".to_string());
        assert_eq!(condition.command, "true");
        assert!(condition.check_exit_code);
        assert!(condition.target_output.is_none());
    }

    #[test]
    fn test_custom_condition_with_output() {
        let condition = CustomCondition::with_output("echo test".to_string(), "test".to_string());
        assert_eq!(condition.command, "echo test");
        assert!(!condition.check_exit_code);
        assert_eq!(condition.target_output, Some("test".to_string()));
    }

    #[test]
    fn test_custom_condition_from_scheme() {
        let scheme = CustomConditionScheme {
            command: "test -f /etc/hosts".to_string(),
            check_exit_code: Some(true),
            target_output: None,
        };
        let condition = CustomCondition::from_scheme(scheme);
        assert_eq!(condition.command, "test -f /etc/hosts");
        assert!(condition.check_exit_code);
    }

    #[test]
    fn test_sync_condition_exit_code() {
        // This should succeed because 'true' returns exit code 0
        let result = sync_condition("true", true, None);
        assert!(result);

        // This should fail because 'false' returns exit code 1
        let result = sync_condition("false", true, None);
        assert!(!result);
    }
}
