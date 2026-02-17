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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new("echo hello".to_string());
        assert_eq!(task.command, "echo hello");
    }

    #[test]
    fn test_task_run() {
        // This test just ensures that the run method can be called without crashing
        let task = Task::new("echo test".to_string());
        task.run();
        assert_eq!(task.command, "echo test");
    }
}
