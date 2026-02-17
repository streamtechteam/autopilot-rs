use std::{thread::sleep, time::Duration};

use colored::Colorize;
use log::{error, info};
use serde::{Deserialize, Serialize};
use tokio_cron_scheduler::JobScheduler;

use crate::{
    conditions::{Condition, ConditionScheme},
    status::{JobStatusEnum, set::set_state_item},
    task::{self, TaskScheme},
    time::{When, add::add_job},
};

pub mod get;
pub mod set;

#[derive(Clone)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub status: JobStatusEnum,
    pub description: String,
    pub when: Option<When>,
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
        when: Option<When>,
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

        let when = scheme.when;
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
            // let scheduler_clone = scheduler.clone();
            // let job_clone = self.clone();
            let _result = add_job(self, &scheduler, run_job).await;
            match _result {
                Err(error) => {
                    error!("{} : {}", self.name, error);
                }
                Ok(_) => {}
            }
        };

        // if let Err(e) = set_state_item(self.id.clone(), JobStatusEnum::) {
        //     error!("Failed to set state item: {}", e);
        // }
    }
}

pub fn run_job(job: Job) {
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
    when: Option<When>,
    check_interval: Option<String>,
    conditions: Vec<ConditionScheme>,
    tasks: Vec<TaskScheme>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::conditions::AlwaysTrueCondition;
    use crate::task::Task;

    #[test]
    fn test_job_creation() {
        let conditions: Vec<Box<dyn Condition>> = vec![Box::new(AlwaysTrueCondition {})];
        let tasks: Vec<Task> = vec![Task::new("echo test".to_string())];
        
        let job = Job::new(
            "test_id".to_string(),
            "test_name".to_string(),
            "test_description".to_string(),
            Some("1000".to_string()),
            None,
            conditions,
            tasks,
        );
        
        assert_eq!(job.id, "test_id");
        assert_eq!(job.name, "test_name");
        assert_eq!(job.description, "test_description");
        assert_eq!(job.check_interval, Some("1000".to_string()));
        assert!(job.conditions.len() == 1);
        assert!(job.tasks.len() == 1);
    }

    #[test]
    fn test_job_from_scheme() {
        let condition_scheme = ConditionScheme::AlwaysTrue(crate::conditions::AlwaysTrueConditionScheme::default());
        let task_scheme = crate::task::TaskScheme {
            command: "echo test".to_string(),
        };
        
        let job_scheme = JobScheme {
            id: "test_id".to_string(),
            name: Some("test_name".to_string()),
            description: Some("test_description".to_string()),
            when: None,
            check_interval: Some("1000".to_string()),
            conditions: vec![condition_scheme],
            tasks: vec![task_scheme],
        };
        
        let job = Job::from_scheme(job_scheme);
        
        assert_eq!(job.id, "test_id");
        assert_eq!(job.name, "test_name");
        assert_eq!(job.description, "test_description");
        assert_eq!(job.check_interval, Some("1000".to_string()));
        assert!(job.conditions.len() == 1);
        assert!(job.tasks.len() == 1);
    }
}
