use serde::{Deserialize, Serialize};

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
    pub fn run(&self) {
        runner::sync_run(self);
    }
}

pub type TaskScheme = Task;
