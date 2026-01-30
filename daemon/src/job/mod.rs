use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_cron_scheduler::JobScheduler;

use crate::{
    conditions::{Condition, ConditionScheme},
    tasks::{self, Task, TaskScheme},
    time::{DateTimeScheme, init::add_job, to_datatime},
};

pub mod get;
// #[derive(Clone)]
pub struct Job {
    pub id: u32,
    pub name: String,
    pub description: String,
    pub when: Option<DateTime<Local>>,
    pub conditions: Vec<Box<dyn Condition>>,
    pub tasks: Vec<tasks::Task>,
}

impl Job {
    pub fn new(
        id: u32,
        name: String,
        description: String,
        when: Option<DateTime<Local>>,
        conditions: Vec<Box<dyn Condition>>,
        tasks: Vec<tasks::Task>,
    ) -> Self {
        Job {
            id,
            name,
            description,
            when,
            conditions,
            tasks,
        }
    }

    pub fn from_scheme(scheme: JobScheme, scheduler: &JobScheduler) -> Self {
        let conditions: Vec<Box<dyn Condition>> = scheme
            .conditions
            .into_iter()
            .map(|condition_scheme| condition_scheme.to_condition(scheduler))
            .collect();

        let tasks: Vec<tasks::Task> = scheme
            .tasks
            .into_iter()
            .map(|task_scheme| tasks::Task::new(task_scheme.command))
            .collect();
        let when: Option<DateTime<Local>> = to_datatime(scheme.when);
        Job {
            id: scheme.id.parse().unwrap_or(0), // Convert string ID to u32
            name: scheme.name,
            description: scheme.description,
            when,
            conditions,
            tasks,
        }
    }

    pub fn from_json(json_str: &str, scheduler: &JobScheduler) -> Result<Self, serde_json::Error> {
        let scheme: JobScheme = serde_json::from_str(json_str)?;
        Ok(Job::from_scheme(scheme, scheduler))
    }

    pub fn add_condition(&mut self, condition: Box<dyn Condition>) {
        self.conditions.push(condition);
    }

    pub fn run(&self, scheduler: &JobScheduler) {
        let mut result = true;
        if (self.when.is_none()) {
            for condition in &self.conditions {
                let condition_result = condition.check();
                // condition.run();
                result = result && condition_result;

                // return;
            }
            if result {
                for task in &self.tasks {
                    task.run();
                }
            }
        } else if self.when.is_some() {
            add_job(self, scheduler, run_job);
        };
    }
}

// impl Clone for Job {
//     fn clone(&self) -> Self {
//         Job {
//             id: self.id,
//             name: self.name.clone(),
//             description: self.description.clone(),
//             when: self.when.clone(),
//             conditions: self.conditions,
//             tasks: self.tasks.clone(),
//         }
//     }
// }

pub fn run_job(job: &Job) {
    let mut result = true;
    for condition in &job.conditions {
        let condition_result = condition.check();
        // condition.run();
        result = result && condition_result;

        // return;
    }
    if result {
        for task in &job.tasks {
            task.run();
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct JobScheme {
    id: String,
    name: String,
    description: String,
    when: DateTimeScheme,
    conditions: Vec<ConditionScheme>,
    tasks: Vec<TaskScheme>,
}

// struct Condition {
//     id: u32,
//     name: String,
//     description: String,
//     job_id: u32,
// }

// impl Condition {
//     pub fn new(id: u32, name: String, description: String, job_id: u32) -> Self {
//         Condition {
//             id,
//             name,
//             description,
//             job_id,
//         }
//     }
// }
