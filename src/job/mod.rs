use std::{thread::sleep, time::Duration};

use chrono::{DateTime, Local};
use colored::Colorize;
use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio_cron_scheduler::JobScheduler;

use crate::{
    conditions::{Condition, ConditionScheme},
    cron::{DateTimeScheme, add::add_job, to_datatime},
    status::{JobStatusEnum, set::set_state_item},
    task::{self, TaskScheme},
};

pub mod get;
pub mod set;

#[derive(Clone)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub status: JobStatusEnum,
    pub description: String,
    pub when: Option<DateTime<Local>>,
    pub check_interval: Option<String>,
    pub conditions: Vec<Box<dyn Condition>>,
    pub tasks: Vec<task::Task>,
}

impl Job {
    pub fn new(
        id: String,
        name: String,
        description: String,
        check_interval: Option<String>,
        when: Option<DateTime<Local>>,
        conditions: Vec<Box<dyn Condition>>,
        tasks: Vec<task::Task>,
    ) -> Self {
        Job {
            id,
            name,
            status: JobStatusEnum::Pending,
            description,
            when,
            check_interval,
            conditions,
            tasks,
        }
    }

    pub fn from_scheme(scheme: JobScheme) -> Self {
        let conditions: Vec<Box<dyn Condition>> = scheme
            .conditions
            .into_iter()
            .map(|condition_scheme| condition_scheme.to_condition())
            .collect();

        let tasks: Vec<task::Task> = scheme
            .tasks
            .into_iter()
            .map(|task_scheme| task::Task::new(task_scheme.command))
            .collect();

        let when = match scheme.when {
            Some(value) => Some(to_datatime(value).expect("Failed to parse when property")),
            None => None,
        };
        Job {
            id: scheme.id.clone(),
            name: scheme.name.unwrap_or(format!("job_{}", scheme.id)),
            status: JobStatusEnum::Unknown,
            description: scheme.description.unwrap_or(" ".to_string()),
            when,
            check_interval: scheme.check_interval,
            conditions,
            tasks,
        }
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let scheme: JobScheme = serde_json::from_str(json_str)?;
        Ok(Job::from_scheme(scheme))
    }

    pub fn add_condition(&mut self, condition: Box<dyn Condition>) {
        self.conditions.push(condition);
    }

    pub async fn run(&self, scheduler: &JobScheduler, quiet: bool) {
        if !quiet {
            info!("{} : {}", "Running job".yellow(), self.name);
        }

        if let Err(e) = set_state_item(self.id.clone(), JobStatusEnum::Running) {
            error!("Failed to set state item: {}", e);
        }

        if self.when.is_none() {
            loop {
                let mut result = true;
                for condition in &self.conditions {
                    let condition_result = condition.check();
                    result = result && condition_result;
                }
                if result {
                    for task in &self.tasks {
                        task.run();
                    }
                    if let Err(e) = set_state_item(self.id.clone(), JobStatusEnum::Success) {
                        error!("Failed to set state item: {}", e);
                    }
                    if !quiet {
                        info!("{} : {}", "Job Completed".green(), self.name);
                    }
                    break;
                } else if !result && self.check_interval.is_some() {
                    let interval_ms = match self
                        .check_interval
                        .as_ref()
                        .expect("WTF you should NOT be seeing this")
                        .parse::<u64>()
                    {
                        Ok(ms) => ms,
                        Err(_) => {
                            error!("check_interval value is not valid, using 1000ms as default");
                            1000
                        }
                    };
                    sleep(Duration::from_millis(interval_ms));
                    continue;
                }
                if let Err(e) = set_state_item(self.id.clone(), JobStatusEnum::Unsatisfied) {
                    error!("Failed to set state item: {}", e);
                }
                if !quiet {
                    info!("{} : {}", "Job Unsatisfied".yellow(), self.name);
                }
                break;
            }
        } else if self.when.is_some() {
            // Since add_job is async, we need to spawn it as a task
            let scheduler_clone = scheduler.clone();
            let job_clone = self.clone();
            let _result = add_job(&job_clone, &scheduler_clone, run_job).await;
            match _result {
                Err(error) => {
                    error!("{} : {}", job_clone.name, error);
                }
                Ok(_) => {}
            }
        };

        // if let Err(e) = set_state_item(self.id.clone(), JobStatusEnum::) {
        //     error!("Failed to set state item: {}", e);
        // }
    }
}

pub fn run_job(job: &Job) {
    let mut result = true;
    for condition in &job.conditions {
        let condition_result = condition.check();
        result = result && condition_result;
    }
    if result {
        for task in &job.tasks {
            task.run();
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct JobScheme {
    id: String,
    name: Option<String>,
    description: Option<String>,
    when: Option<DateTimeScheme>,
    check_interval: Option<String>,
    conditions: Vec<ConditionScheme>,
    tasks: Vec<TaskScheme>,
}
