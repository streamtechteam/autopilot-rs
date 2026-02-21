use colored::Colorize;
use log::{error, info};
use tokio_cron_scheduler::JobScheduler;

use crate::{
    cli::status::check_if_running,
    error::AutoPilotError,
    job::{Job, get::get_jobs},
    logging::init_logging,
    status::set::set_status_initial,
    time::init::init_time_check,
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
    pub async fn reload_config(&mut self) {
        info!("{}", "Reloading Autopilot...".yellow());

        let new_jobs = get_jobs(false);

        // Stop all existing jobs
        self.scheduler
            .shutdown()
            .await
            .expect("Failed to shutdown scheduler");

        // Recreate scheduler
        self.scheduler = init_time_check().await.expect("Failed to reinit cron");

        // Update jobs
        self.jobs = new_jobs;

        // Reload all jobs
        self.run_jobs();

        info!("{}", "Autopilot reloaded successfully!".green())
    }
    pub fn start(&mut self, verbose: bool) {
        Self::prepare_logging(verbose);
        if Self::check_instance() {
            return;
        }
        Self::init_status().expect("failed to init status");
        self.load_jobs();
        info!("{}", "Autopilot served!".green());
        self.run_jobs();
    }
    fn check_instance() -> bool {
        match check_if_running() {
            true => {
                error!("there is already an instance of Autopilot running");
                true
            }
            _ => {
                false
            }
        }
    }
    fn init_status() -> Result<(), AutoPilotError> {
        if let Err(e) = set_status_initial() {
            error!("Failed to initialize status: {}", e);
            return Err(AutoPilotError::State(e));
        }
        Ok(())
    }
    fn run_jobs(&mut self) {
        for job in self.jobs.clone() {
            let scheduler = self.scheduler.clone();
            tokio::task::spawn(async move {
                job.run(&scheduler, false).await;
            });
        }
    }
    fn load_jobs(&mut self) {
        self.jobs = get_jobs(false);
    }
    // fn add_job(&mut self, job: Job) {
    //     self.jobs.push(job);
    // }
    // fn remove_job(&mut self, job: Job) {
    //     // self.jobs.remove(self.jobs.iter().index)
    // }
    pub fn prepare_logging(verbose: bool) {
        init_logging(verbose);
    }
}
