use colored::Colorize;
use log::{info, warn};
use tokio::task::JoinHandle;
use tokio_cron_scheduler::JobScheduler;

use crate::health::check_if_running;
use autopilot_core_error::AutoPilotError;
use autopilot_core_job::{Job, get};
use autopilot_core_logging::init_logging;
use autopilot_core_status::{JobStatusEnum, StateManager};
use autopilot_core_time::init::init_time_check;

pub struct AutoPilot {
    pub started: bool,
    pub scheduler: JobScheduler,
    pub jobs: Vec<Job>,
    pub _jobs_handles: Vec<JoinHandle<()>>,
    pub status_manager: StateManager,
}

impl AutoPilot {
    pub async fn new() -> Result<Self, AutoPilotError> {
        let scheduler = init_time_check().await?;

        Ok(Self {
            started: false,
            scheduler,
            jobs: Vec::new(),
            _jobs_handles: Vec::new(),
            status_manager: StateManager::new(),
        })
    }
    pub async fn init(&mut self, verbose: bool) -> Result<(), AutoPilotError> {
        init_logging(verbose);
        if check_if_running() {
            return Err(AutoPilotError::Autopilot(
                "Instance already running".to_string(),
            ));
        }
        self.load_jobs().await?;
        self.init_status().await?;
        Ok(())
    }
    pub async fn init_status(&mut self) -> Result<(), AutoPilotError> {
        let entries: Vec<(String, String)> = self
            .jobs
            .iter()
            .map(|job| (job.id.clone(), job.name.clone()))
            .collect();

        let status_manager = StateManager::new();
        for (id, name) in entries {
            status_manager
                .set(id, Some(name), JobStatusEnum::Unknown)
                .await;
        }
        self.status_manager = status_manager;
        Ok(())
    }
    pub async fn reload(&mut self) -> Result<(), AutoPilotError> {
        let was_started = self.started;
        self.stop_jobs().await?;
        info!("{}", "Reloading Autopilot...".yellow());
        self.load_jobs().await?;
        self.init_status().await?;
        if was_started {
            self.start_all();
        }

        info!("{}", "Autopilot reloaded successfully!".green());
        Ok(())
    }
    pub async fn restart(&mut self) -> Result<(), AutoPilotError> {
        self.stop_jobs().await?;
        self.load_jobs().await?;
        self.init_status().await?;
        self.start_all();

        Ok(())
    }
    pub fn start_all(&mut self) {
        self._jobs_handles = self.run_jobs();
        self.started = true;
        info!("{}", "Autopilot served!".green());
    }

    pub fn run_jobs(&mut self) -> Vec<JoinHandle<()>> {
        self.jobs.iter_mut().for_each(|job| {
            job.loaded = true;
        });

        let mut handles = vec![];
        for mut job in self.jobs.clone() {
            let scheduler = self.scheduler.clone();
            let sm = self.status_manager.clone();
            handles.push(tokio::task::spawn(async move {
                job.run(sm, &scheduler, false).await;
            }))
        }

        handles
    }
    pub async fn load_jobs(&mut self) -> Result<(), AutoPilotError> {
        self.jobs = tokio::task::spawn_blocking(|| get::get_jobs(false))
            .await
            .map_err(|e| AutoPilotError::JobExecution(format!("Failed to load jobs: {}", e)))??;
        Ok(())
    }
    /// Stop running jobs and clear the scheduler's pending work.
    pub async fn stop_jobs(&mut self) -> Result<(), AutoPilotError> {
        warn!("Stopping jobs...");
        self._jobs_handles.iter().for_each(|handle| {
            handle.abort();
        });
        self._jobs_handles.clear();
        self.started = false;

        self.scheduler
            .shutdown()
            .await
            .map_err(|e| AutoPilotError::JobExecution(e.to_string()))?;
        self.scheduler = init_time_check().await?;

        Ok(())
    }
    pub async fn shutdown(&mut self) -> Result<(), AutoPilotError> {
        self.stop_jobs().await?;
        self.scheduler
            .shutdown()
            .await
            .map_err(|e| AutoPilotError::JobExecution(e.to_string()))
    }
}

#[cfg(test)]
#[path = "autopilot.spec.rs"]
mod spec;

//  TODO
// need to consider somethings regarding the structure of AutoPilot
