use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use autopilot_core_error::AutoPilotError;

pub mod runner;
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Task")]
pub struct Task {
    pub command: String,
    /// Optional timeout in seconds.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = "Timeout",
        range(min = 1),
        extend("x-autopilot" = {"unit": "seconds", "group": "task"})
    )]
    pub timeout: Option<u64>,
}

impl Task {
    pub fn new(command: String) -> Self {
        Task {
            command,
            timeout: None,
        }
    }

    pub fn with_timeout(mut self, timeout: u64) -> Self {
        self.timeout = Some(timeout);
        self
    }

    pub fn run(&self) -> JoinHandle<Result<(), AutoPilotError>> {
        let command = self.command.clone();
        let timeout = self.timeout;
        tokio::task::spawn(async move { runner::async_run(command, timeout).await })
    }
    pub fn run_sync(&self) -> Result<(), AutoPilotError> {
        runner::sync_run(self.command.clone())?;
        Ok(())
    }
}

pub type TaskScheme = Task;

#[cfg(test)]
#[path = "task.spec.rs"]
mod spec;
