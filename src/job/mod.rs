use chrono::{DateTime, Local};
use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio;
use tokio_cron_scheduler::JobScheduler;

use crate::{
    conditions::{Condition, ConditionScheme},
    cron::{DateTimeScheme, add::add_job, to_datatime},
    state::{Status, get::get_status_log, set::set_state_item},
    tasks::{self, TaskScheme},
};

pub mod get;
// #[derive(Clone)]
#[derive(Clone)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub status: Status,
    pub description: String,
    pub when: Option<DateTime<Local>>,
    pub conditions: Vec<Box<dyn Condition>>,
    pub tasks: Vec<tasks::Task>,
}

impl Job {
    pub fn new(
        id: String,
        name: String,
        description: String,
        when: Option<DateTime<Local>>,
        conditions: Vec<Box<dyn Condition>>,
        tasks: Vec<tasks::Task>,
    ) -> Self {
        Job {
            id,
            name,
            status: Status::Pending,
            description,
            when,
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

        let tasks: Vec<tasks::Task> = scheme
            .tasks
            .into_iter()
            .map(|task_scheme| tasks::Task::new(task_scheme.command))
            .collect();

        let when = match scheme.when {
            Some(value) => to_datatime(value),
            None => None,
        };
        Job {
            id: scheme.id.clone(),
            name: scheme.name.unwrap_or(format!("job_{}", scheme.id)),
            status: Status::Unknown,
            description: scheme.description.unwrap_or(" ".to_string()),
            when,
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

    pub async fn run(&self, scheduler: &JobScheduler) {
        println!("Running job: {}", self.name);
        set_state_item(self.id.clone(), Status::Running).expect("failed to set state item");

        let mut result = true;
        if self.when.is_none() {
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
            // Since add_job is async, we need to spawn it as a task
            let scheduler_clone = scheduler.clone();
            let job_clone = self.clone();
            // tokio::spawn(async move {
            let _result = add_job(&job_clone, &scheduler_clone, run_job).await;
            match _result {
                Err(error) => {
                    error!("{} : {}", job_clone.name, error);
                }
                Ok(_) => {
                    // info!("job successfull")
                }
            }
            // });
        };
        println!("Job completed: {}", self.name);
        set_state_item(self.id.clone(), Status::Completed).expect("failed to set state item");
    }
}

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
    name: Option<String>,
    description: Option<String>,
    when: Option<DateTimeScheme>,
    conditions: Vec<ConditionScheme>,
    tasks: Vec<TaskScheme>,
}
