use std::time::{Duration, Instant};

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
pub mod interval;
pub mod repository;
pub mod schedule;
pub mod set;

use interval::effective_check_interval;
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
    /// Default polling interval in milliseconds for all conditions.
    /// Each condition may override this with its own `check_interval`.
    pub check_interval: Option<String>,
    pub conditions: Vec<Box<dyn Condition>>,
    pub tasks: Vec<Task>,
    pub task_mode: TaskMode,
    pub loaded: bool,
}

/// Per-condition polling state: the condition plus its cached result and next due time.
type ConditionState = (Box<dyn Condition>, Option<bool>, Option<Instant>);

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
            let global_interval = self.check_interval.as_deref();
            // Per-condition polling state: the last result plus the next due
            // time. Conditions without an effective interval are checked once
            // and their result is reused.
            let mut states: Vec<ConditionState> = self
                .conditions
                .iter()
                .map(|condition| (condition.clone(), None, None))
                .collect();

            loop {
                let mut all_satisfied = true;
                let mut next_wakeup: Option<Instant> = None;
                let mut any_polling = false;

                for (condition, last_result, next_due) in states.iter_mut() {
                    let interval = effective_check_interval(condition.as_ref(), global_interval);
                    let due = if last_result.is_none() {
                        true
                    } else if let (Some(_), Some(due_at)) = (interval, *next_due) {
                        Instant::now() >= due_at
                    } else {
                        interval.is_some()
                    };

                    if due {
                        let check = condition.clone();
                        // Run the heavy check on the blocking pool
                        let result = tokio::task::spawn_blocking(move || check.check())
                            .await
                            .expect("condition check panicked");
                        *last_result = Some(result);
                        *next_due = interval.map(|wait| Instant::now() + wait);
                    }

                    all_satisfied &= last_result.unwrap_or(false);
                    if interval.is_some() {
                        any_polling = true;
                    }
                    if let Some(due_at) = *next_due {
                        let earliest = next_wakeup.map_or(due_at, |w| w.min(due_at));
                        next_wakeup = Some(earliest);
                    }
                }

                if all_satisfied {
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
                } else if any_polling {
                    // Sleep until the next condition is due. The 1ms floor
                    // keeps a 0ms interval from busy-looping.
                    let delay = next_wakeup
                        .map(|wakeup| wakeup.saturating_duration_since(Instant::now()))
                        .unwrap_or(Duration::from_millis(1))
                        .max(Duration::from_millis(1));
                    tokio::time::sleep(delay).await;
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
