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
}

impl Task {
    pub fn new(command: String) -> Self {
        Task { command }
    }
    pub fn run(&self) -> JoinHandle<Result<(), AutoPilotError>> {
        let command = self.command.clone();
        tokio::task::spawn(async move { runner::async_run(command).await })
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
