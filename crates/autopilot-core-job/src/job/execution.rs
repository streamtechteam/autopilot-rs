use std::time::Duration;

use colored::Colorize;
use log::{error, info};
use tokio_cron_scheduler::JobScheduler;

use autopilot_core_status::{JobStatusEnum, StateManager};

use super::Job;
use super::conditions::all_conditions_met;
use super::schedule::add_job;
use super::task_mode::run_tasks;

impl Job {
    pub async fn run(
        &mut self,
        state_manager: StateManager,
        scheduler: &JobScheduler,
        quiet: bool,
    ) {
        if !quiet {
            info!("{} : {}", "Running job".yellow(), self.name);
        }

        state_manager
            .set(self.id.clone(), None, JobStatusEnum::Running)
            .await;

        if self.when.is_none() {
            loop {
                if all_conditions_met(&self.conditions).await {
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
        } else {
            state_manager
                .set(self.id.clone(), None, JobStatusEnum::Scheduled)
                .await;
            let result = add_job(self, scheduler, run_job, state_manager.clone()).await;
            if let Err(error) = result {
                error!("{} : {}", self.name, error);
            }
        }
    }
}

pub async fn run_job(job: Job, state_manager: StateManager) {
    state_manager
        .set(job.id.clone(), None, JobStatusEnum::Running)
        .await;

    if all_conditions_met(&job.conditions).await {
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
