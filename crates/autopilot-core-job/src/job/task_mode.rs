use futures::future::join_all;
use log::error;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::task::{JoinError, JoinHandle};

use autopilot_core_error::AutoPilotError;
use autopilot_core_task::Task;

/// How the tasks in a job are executed.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[schemars(title = "Task mode")]
pub enum TaskMode {
    /// Run all tasks at the same time. This is the default.
    #[default]
    Concurrent,
    /// Run tasks one at a time and continue after a task fails.
    Sequential,
    /// Run tasks one at a time and stop after the first failure.
    SequentialStopOnFailure,
}

/// Runs the tasks in the requested mode and reports whether every task succeeded.
pub async fn run_tasks(tasks: &[Task], mode: TaskMode) -> bool {
    match mode {
        TaskMode::Concurrent => {
            let handles: Vec<JoinHandle<Result<(), AutoPilotError>>> =
                tasks.iter().map(Task::run).collect();
            let mut all_succeeded = true;
            for result in join_all(handles).await {
                all_succeeded &= log_task_result(result);
            }
            all_succeeded
        }
        TaskMode::Sequential | TaskMode::SequentialStopOnFailure => {
            let mut all_succeeded = true;
            for task in tasks {
                let result = task.run().await;
                let succeeded = log_task_result(result);
                all_succeeded &= succeeded;
                if !succeeded && mode == TaskMode::SequentialStopOnFailure {
                    break;
                }
            }
            all_succeeded
        }
    }
}

fn log_task_result(result: Result<Result<(), AutoPilotError>, JoinError>) -> bool {
    match result {
        Ok(Ok(())) => true,
        Ok(Err(err)) => {
            error!("{err}");
            false
        }
        Err(err) => {
            error!("Failed to join task: {err}");
            false
        }
    }
}
