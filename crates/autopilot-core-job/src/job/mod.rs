use std::time::Duration;

use colored::Colorize;
use futures::future::join_all;
use log::{error, info};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use tokio::task::JoinHandle;
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
    //  TODO - IMPORTANT
    // i should add a toggle for tasks running method (concurrent, sequential)
    pub tasks: Vec<Task>,
    pub loaded: bool,
}

impl Job {
    pub fn new(
        id: String,
        name: String,
        description: String,
        check_interval: Option<String>,
        when: Option<When>,
        conditions: Vec<Box<dyn Condition>>,
        tasks: Vec<Task>,
    ) -> Self {
        Job {
            id,
            name,
            description,
            when,
            check_interval,
            conditions,
            tasks,
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
                    run_tasks(&self.tasks).await;
                    state_manager
                        .set(self.id.clone(), None, JobStatusEnum::Completed)
                        .await;
                    if !quiet {
                        info!("{} : {}", "Job Completed".green(), self.name);
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
        run_tasks(&job.tasks).await;
        state_manager
            .set(job.id.clone(), None, JobStatusEnum::Completed)
            .await;
    } else {
        state_manager
            .set(job.id, None, JobStatusEnum::Unsatisfied)
            .await;
    }
}

pub async fn run_tasks(tasks: &[Task]) {
    let mut handles: Vec<JoinHandle<Result<(), AutoPilotError>>> = vec![];
    for task in tasks {
        handles.push(task.run());
    }
    join_all(handles)
        .await
        .iter()
        .for_each(|handle| match handle {
            Ok(handle) => match handle {
                Ok(_) => {}
                Err(err) => {
                    error!("{}", err)
                }
            },
            Err(err) => {
                error!("Failed to join handles : {}", err);
            }
        });
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
}

#[cfg(test)]
#[path = "job.spec.rs"]
mod spec;

#[cfg(test)]
pub(crate) mod test_support;

//  TODO
// this module is getting very long and big, can probably benefit some small improvments and refactors
