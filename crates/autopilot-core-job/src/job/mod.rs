use std::time::Duration;

use colored::Colorize;
use futures::future::join_all;
use log::{error, info};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::task::{JoinError, JoinHandle};
use tokio_cron_scheduler::JobScheduler;

use autopilot_core_conditions::{Condition, ConditionScheme};
use autopilot_core_error::AutoPilotError;
use autopilot_core_status::{JobStatusEnum, StateManager};
use autopilot_core_task::{self, Task, TaskScheme};
use autopilot_core_time::When;

pub mod get;
pub mod repository;
pub mod schedule;
pub mod set;

use schedule::add_job;

pub use repository::{FsJobRepository, JobRepository};

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

#[derive(Clone)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub description: String,
    pub when: Option<When>,
    //  TODO
    // i should probably make check_interval condition specific
    pub check_interval: Option<String>,
    pub conditions: Vec<Box<dyn Condition>>,
    pub tasks: Vec<Task>,
    pub task_mode: TaskMode,
    pub loaded: bool,
}

impl Job {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        name: String,
        description: String,
        check_interval: Option<String>,
        when: Option<When>,
        conditions: Vec<Box<dyn Condition>>,
        tasks: Vec<Task>,
        task_mode: TaskMode,
    ) -> Self {
        Job {
            id,
            name,
            description,
            when,
            check_interval,
            conditions,
            tasks,
            task_mode,
            loaded: false,
        }
    }

    pub fn from_scheme(scheme: JobScheme) -> Self {
        let conditions: Vec<Box<dyn Condition>> = scheme
            .conditions
            .into_iter()
            .map(|condition_scheme| condition_scheme.to_condition())
            .collect();

        let tasks: Vec<Task> = scheme
            .tasks
            .into_iter()
            .map(|task_scheme| Task::new(task_scheme.command))
            .collect();

        let when = scheme.when;
        Job {
            id: scheme.id.clone(),
            name: scheme.name.unwrap_or(format!("job_{}", scheme.id)),
            description: scheme.description.unwrap_or(" ".to_string()),
            when,
            check_interval: scheme.check_interval,
            conditions,
            tasks,
            task_mode: scheme.task_mode,
            loaded: false,
        }
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let scheme: JobScheme = serde_json::from_str(json_str)?;
        Ok(Job::from_scheme(scheme))
    }

    pub fn add_condition(&mut self, condition: Box<dyn Condition>) {
        self.conditions.push(condition);
    }
    pub async fn run(
        &mut self,
        state_manager: StateManager,
        scheduler: &JobScheduler,
        quiet: bool,
    ) {
        if !quiet {
            info!("{} : {}", "Running job".yellow(), self.name);
        }
        // self.status = JobStatusEnum::Running;

        state_manager
            .set(self.id.clone(), None, JobStatusEnum::Running)
            .await;

        if self.when.is_none() {
            loop {
                let mut result = true;
                for condition in self.conditions.clone() {
                    // Clone the whole vector per iteration (if conditions are Clone)
                    // Run the heavy check on the blocking pool
                    let condition_result = tokio::task::spawn_blocking(move || condition.check())
                        .await
                        .expect("condition check panicked");
                    result = result && condition_result;
                }
                if result {
                    let all_tasks_succeeded = run_tasks(&self.tasks, self.task_mode).await;
                    if all_tasks_succeeded {
                        state_manager
                            .set(self.id.clone(), None, JobStatusEnum::Completed)
                            .await;
                        if !quiet {
                            info!("{} : {}", "Job Completed".green(), self.name);
                        }
                    } else {
                        state_manager
                            .set(self.id.clone(), None, JobStatusEnum::Failed)
                            .await;
                        if !quiet {
                            info!("{} : {}", "Job Failed".red(), self.name);
                        }
                    }
                    break;
                } else if let Some(interval) = &self.check_interval {
                    let interval_ms = interval.parse::<u64>().unwrap_or_else(|_| {
                        error!("check_interval value is not valid, using 1000ms as default");
                        1000
                    });
                    tokio::time::sleep(Duration::from_millis(interval_ms)).await;
                } else {
                    state_manager
                        .set(self.id.clone(), None, JobStatusEnum::Unsatisfied)
                        .await;
                    if !quiet {
                        info!("{} : {}", "Job Unsatisfied".yellow(), self.name);
                    }
                    break;
                }
            }
        } else if self.when.is_some() {
            // self.status = JobStatusEnum::Scheduled;
            state_manager
                .set(self.id.clone(), None, JobStatusEnum::Scheduled)
                .await;
            let _result = add_job(self, scheduler, run_job, state_manager.clone()).await;
            if let Err(error) = _result {
                error!("{} : {}", self.name, error);
            }
        };
    }
}

pub async fn run_job(job: Job, state_manager: StateManager) {
    state_manager
        .set(job.id.clone(), None, JobStatusEnum::Running)
        .await;

    // Offload every condition.check() to the blocking pool, run in parallel
    let mut condition_futures = Vec::new();
    for condition in job.conditions {
        let fut = tokio::task::spawn_blocking(move || condition.check());
        condition_futures.push(fut);
    }

    let results = futures::future::join_all(condition_futures).await;
    let all_ok = results.into_iter().all(|res| res.unwrap_or(false));

    if all_ok {
        let all_tasks_succeeded = run_tasks(&job.tasks, job.task_mode).await;
        let status = if all_tasks_succeeded {
            JobStatusEnum::Completed
        } else {
            JobStatusEnum::Failed
        };
        state_manager.set(job.id.clone(), None, status).await;
    } else {
        state_manager
            .set(job.id, None, JobStatusEnum::Unsatisfied)
            .await;
    }
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

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "AutoPilot job file")]
pub struct JobScheme {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub when: Option<When>,
    pub check_interval: Option<String>,
    pub conditions: Vec<ConditionScheme>,
    pub tasks: Vec<TaskScheme>,
    #[serde(default)]
    pub task_mode: TaskMode,
}

#[cfg(test)]
#[path = "job.spec.rs"]
mod spec;

#[cfg(test)]
pub(crate) mod test_support;

//  TODO
// this module is getting very long and big, can probably benefit some small improvments and refactors
