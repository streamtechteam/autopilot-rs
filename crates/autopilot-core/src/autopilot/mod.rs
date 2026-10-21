use colored::Colorize;
use log::{info, warn};
use tokio::task::JoinHandle;
use tokio_cron_scheduler::JobScheduler;

use crate::{
    checks::check_if_running,
    error::AutoPilotError,
    job::{Job, get::get_jobs},
    logging::init_logging,
    status::{JobStatusEnum, StateManager},
    time::init::init_time_check,
};

// #[derive(Clone)]
pub struct AutoPilot {
    pub started: bool,
    pub scheduler: JobScheduler,
    pub jobs: Vec<Job>,
    pub jobs_handles: Vec<JoinHandle<()>>,
    pub status_manager: StateManager,
}

impl AutoPilot {
    pub async fn new() -> Result<Self, AutoPilotError> {
        let status_manager = StateManager::new();
        // Initialise entries for existing jobs
        for job in get_jobs(true) {
            status_manager
                .set(job.id, Some(job.name), JobStatusEnum::Unknown)
                .await;
        }

        let scheduler = init_time_check().await?;

        Ok(Self {
            started: false,
            scheduler,
            jobs: Vec::new(),
            jobs_handles: Vec::new(),
            status_manager: status_manager,
        })
    }

    pub async fn init(&mut self, verbose: bool) -> Result<(), AutoPilotError> {
        Self::prepare_logging(verbose);
        if Self::check_instance() {
            return Err(AutoPilotError::Autopilot(
                "Instance already running".to_string(),
            ));
        }
        self.init_status().await?;
        self.load_jobs();
        Ok(())
    }
    pub async fn reload(&mut self) -> Result<(), AutoPilotError> {
        self.stop_jobs().await?;
        info!("{}", "Reloading Autopilot...".yellow());
        self.init_status().await?;
        self.load_jobs();

        self.start(false);

        info!("{}", "Autopilot reloaded successfully!".green());
        Ok(())
    }
    pub fn start(&mut self, _: bool) {
        self.jobs_handles = self.run_jobs();
        info!("{}", "Autopilot served!".green());
    }
    pub fn check_instance() -> bool {
        check_if_running()
    }
    pub async fn init_status(&mut self) -> Result<(), AutoPilotError> {
        let status_manager = StateManager::new();
        // Initialise entries for existing jobs
        for job in get_jobs(true) {
            status_manager
                .set(job.id, Some(job.name), JobStatusEnum::Unknown)
                .await;
        }
        self.status_manager = status_manager;
        Ok(())
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
    pub fn load_jobs(&mut self) {
        self.jobs = get_jobs(false);
    }

    /// Stop all jobs (graceful shutdown of scheduler)
    pub async fn stop_jobs(&mut self) -> Result<(), AutoPilotError> {
        warn!("Stopping jobs...");
        self.jobs_handles.iter().for_each(|handle| {
            handle.abort();
        });
        self.scheduler
            .shutdown()
            .await
            .map_err(|e| AutoPilotError::JobExecution(e.to_string()))?;
        self.jobs = vec![];
        self.jobs_handles = vec![];

        Ok(())
    }
    pub fn prepare_logging(verbose: bool) {
        init_logging(verbose);
    }
    pub async fn shutdown(&mut self) -> Result<(), AutoPilotError> {
        self.stop_jobs().await
    }
}
