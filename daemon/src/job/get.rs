use std::{fs, path::PathBuf};

use colored::*;
use serde_json::Value;
use tokio_cron_scheduler::JobScheduler;

use crate::{
    directory::get_jobs_directory,
    job::{Job, JobScheme},
};

pub fn get_jobs(scheduler: &JobScheduler) -> Vec<Job> {
    let mut jobs_string: Vec<String> = vec![];
    let mut job_objects: Vec<Job> = vec![];
    let jobs_path = get_jobs_paths();
    for job in &jobs_path {
        jobs_string.push(fs::read_to_string(job).expect("Failed to read job file"));
    }

    for (i, job_str) in jobs_string.iter().enumerate() {
        match serde_json::from_str::<JobScheme>(job_str) {
            Ok(job_scheme) => {
                let job_object = Job::from_scheme(job_scheme, scheduler);
                job_objects.push(job_object);
            }
            Err(e) => {
                // Use the index `i` directly instead of searching again
                let job_path = &jobs_path
                    .get(i)
                    .and_then(|p| p.to_str())
                    .unwrap_or("unknown");

                println!(
                    "Failed to parse job: \n Job path: {} \n Error: {}",
                    job_path.green(),
                    e.to_string().red()
                );
            }
        }
    }

    // logger();
    for job in &job_objects {
        println!("Loaded job: {}", job.name);
    }
    job_objects
}

pub fn get_jobs_paths() -> Vec<PathBuf> {
    let path = get_jobs_directory();
    let mut jobs_path: Vec<PathBuf> = vec![];

    for job in fs::read_dir(path).unwrap() {
        jobs_path.push(job.unwrap().path());
    }
    jobs_path
}
