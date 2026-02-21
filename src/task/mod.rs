use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;

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
    pub fn run(&self) -> JoinHandle<()> {
        let command = self.command.clone();
        tokio::task::spawn(async move {
            runner::async_run(command).await;
        })
    }
    pub fn run_sync(&self) {
        runner::sync_run(self.command.clone());
    }
}

pub type TaskScheme = Task;
