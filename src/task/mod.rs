use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

use crate::error::AutoPilotError;

pub mod runner;
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Task {
    pub command: String,
}

impl Task {
    pub fn new(command: String) -> Self {
        Task { command }
    }
    pub fn run(&self) -> JoinHandle<Result<(), AutoPilotError>> {
        let command = self.command.clone();
        tokio::task::spawn(async move {
            return runner::async_run(command).await;
        })
    }
    pub fn run_sync(&self) -> Result<(), AutoPilotError> {
        runner::sync_run(self.command.clone())?;
        Ok(())
    }
}

pub type TaskScheme = Task;
