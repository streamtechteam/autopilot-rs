use std::panic::{self, AssertUnwindSafe};

// use std::thread;
use tokio::{self, signal};
// use futures::{self, };
use serde_json::{self};
use tokio_cron_scheduler::JobScheduler;

use crate::{
    conditions::{output_condition::OutputCondition, variable_condition::VariableCondition},
    directory::get_directory,
    job::get::get_jobs,
    logging::init_logging,
    tasks::Task,
    time::init::{self, init_time_check},
};

mod conditions;
mod directory;
mod job;
mod logging;
mod tasks;
mod time;

struct AutoPilot {
    scheduler: JobScheduler,
}

impl AutoPilot {
    pub async fn new() -> Self {
        Self {
            scheduler: init_time_check().await,
        }
        // self.scheduler =
    }
    pub fn start(&mut self) {
        Self::prepare_logging();
    }
    pub fn prepare_logging() {
        init_logging();
    }
}

#[tokio::main]
async fn main() {
    // let scheduler = init_time_check().await;
    // println!("test");
    let mut auto_pilot = AutoPilot::new().await;
    auto_pilot.start();
    let scheduler = &auto_pilot.scheduler;
    // get_directory();
    // Get jobs from JSON files and run them
    let mut jobs = vec![];
    // let result = panic::catch_unwind(AssertUnwindSafe(|| {
    // could panic
    jobs = get_jobs(scheduler);
    // }));
    // let scheduler = init_time_check().await;

    // match result {
    //     Ok(_) => println!("Getting Jobs completed safely ✅"),
    //     Err(_) => println!("Getting Jobs panicked ❌ but program continues"),
    // }
    for job in jobs {
        let scheduler = scheduler.clone();
        tokio::task::spawn(async move {
            let result = panic::catch_unwind(AssertUnwindSafe(|| {
                job.run(&scheduler); // could panic
            }));

            match result {
                Ok(_) => println!("Job completed safely ✅"),
                Err(_) => println!("Job panicked ❌ but program continues"),
            }
            // job.run();
        });
    }
    // Keep the daemon running until Ctrl+C is pressed
    signal::ctrl_c().await.expect("Failed to listen for Ctrl+C");
    println!("Shutting down daemon...");
}
