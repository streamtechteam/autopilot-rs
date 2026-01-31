use serde::{Deserialize, Serialize};

pub mod runner;
#[derive(Clone)]
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

#[derive(Debug, Serialize, Deserialize)]
pub struct TaskScheme {
    // pub name: String,
    pub command: String,
}
