use tokio_cron_scheduler::JobScheduler;
use std::sync::Arc;

use crate::{
    conditions::Condition,
    job::Job,
    task::Task,
    logging::init_logging,
    time::init::init_time_check,
    error::AutoPilotError,
};

pub struct AutoPilot {
    pub scheduler: JobScheduler,
    pub jobs: Vec<Job>,
}

impl AutoPilot {
    pub async fn new() -> Self {
        Self {
            scheduler: init_time_check().await.expect("failed to init cron"),
            jobs: Vec::new(),
        }
    }

    pub fn add_job(&mut self, job: Job) {
        self.jobs.push(job);
    }

    pub fn remove_job(&mut self, index: usize) -> Result<Job, AutoPilotError> {
        if index >= self.jobs.len() {
            return Err(AutoPilotError::JobNotFound(index));
        }
        Ok(self.jobs.remove(index))
    }

    pub fn get_job(&self, index: usize) -> Result<&Job, AutoPilotError> {
        if index >= self.jobs.len() {
            return Err(AutoPilotError::JobNotFound(index));
        }
        Ok(&self.jobs[index])
    }

    pub fn get_mut_job(&mut self, index: usize) -> Result<&mut Job, AutoPilotError> {
        if index >= self.jobs.len() {
            return Err(AutoPilotError::JobNotFound(index));
        }
        Ok(&mut self.jobs[index])
    }

    pub fn len(&self) -> usize {
        self.jobs.len()
    }

    pub fn is_empty(&self) -> bool {
        self.jobs.is_empty()
    }

    pub fn clear(&mut self) {
        self.jobs.clear();
    }

    pub fn start(&mut self, verbose: bool) {
        Self::prepare_logging(verbose);
    }

    pub fn prepare_logging(verbose: bool) {
        init_logging(verbose);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{conditions::AlwaysTrueCondition, task::Task};

    #[test]
    fn test_autopilot_new() {
        // We can't easily test the async new() function, so we'll test the other methods
        let mut autopilot = AutoPilot {
            scheduler: tokio_cron_scheduler::JobScheduler::new(),
            jobs: Vec::new(),
        };
        
        assert_eq!(autopilot.len(), 0);
        assert!(autopilot.is_empty());
    }

    #[test]
    fn test_autopilot_add_job() {
        let mut autopilot = AutoPilot {
            scheduler: tokio_cron_scheduler::JobScheduler::new(),
            jobs: Vec::new(),
        };
        
        let condition = Box::new(AlwaysTrueCondition {});
        let task = Task::new("echo test".to_string());
        let job = Job::new("test_job".to_string(), condition, vec![task]);
        
        autopilot.add_job(job);
        assert_eq!(autopilot.len(), 1);
        assert!(!autopilot.is_empty());
    }

    #[test]
    fn test_autopilot_remove_job() {
        let mut autopilot = AutoPilot {
            scheduler: tokio_cron_scheduler::JobScheduler::new(),
            jobs: Vec::new(),
        };
        
        let condition = Box::new(AlwaysTrueCondition {});
        let task = Task::new("echo test".to_string());
        let job = Job::new("test_job".to_string(), condition, vec![task]);
        
        autopilot.add_job(job);
        let removed_job = autopilot.remove_job(0).unwrap();
        assert_eq!(removed_job.name, "test_job");
        assert_eq!(autopilot.len(), 0);
    }

    #[test]
    fn test_autopilot_remove_job_invalid_index() {
        let mut autopilot = AutoPilot {
            scheduler: tokio_cron_scheduler::JobScheduler::new(),
            jobs: Vec::new(),
        };
        
        let result = autopilot.remove_job(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_autopilot_get_job() {
        let mut autopilot = AutoPilot {
            scheduler: tokio_cron_scheduler::JobScheduler::new(),
            jobs: Vec::new(),
        };
        
        let condition = Box::new(AlwaysTrueCondition {});
        let task = Task::new("echo test".to_string());
        let job = Job::new("test_job".to_string(), condition, vec![task]);
        
        autopilot.add_job(job);
        let retrieved_job = autopilot.get_job(0).unwrap();
        assert_eq!(retrieved_job.name, "test_job");
    }

    #[test]
    fn test_autopilot_get_job_invalid_index() {
        let autopilot = AutoPilot {
            scheduler: tokio_cron_scheduler::JobScheduler::new(),
            jobs: Vec::new(),
        };
        
        let result = autopilot.get_job(0);
        assert!(result.is_err());
    }

    #[test]
    fn test_autopilot_clear() {
        let mut autopilot = AutoPilot {
            scheduler: tokio_cron_scheduler::JobScheduler::new(),
            jobs: Vec::new(),
        };
        
        let condition = Box::new(AlwaysTrueCondition {});
        let task = Task::new("echo test".to_string());
        let job = Job::new("test_job".to_string(), condition, vec![task]);
        
        autopilot.add_job(job);
        assert_eq!(autopilot.len(), 1);
        
        autopilot.clear();
        assert_eq!(autopilot.len(), 0);
        assert!(autopilot.is_empty());
    }
}
