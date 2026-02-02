use std::{fs, path::PathBuf};

use colored::*;
use log::info;
use tokio_cron_scheduler::JobScheduler;

use crate::{
    directory::get_jobs_path,
    job::{Job, JobScheme},
    utilities::jsonc_parser::jsonc_parse,
};

pub fn get_jobs(quiet: bool) -> Vec<Job> {
    let mut jobs_string: Vec<String> = vec![];
    let mut job_objects: Vec<Job> = vec![];
    let jobs_path = get_jobs_paths();
    for job in &jobs_path {
        jobs_string.push(fs::read_to_string(job).expect("Failed to read job file"));
    }

    for (i, job_str) in jobs_string.iter().enumerate() {
        match serde_json::from_str::<JobScheme>(jsonc_parse(job_str).as_str()) {
            Ok(job_scheme) => {
                let job_object = Job::from_scheme(job_scheme);
                if !quiet {
                    info!("Loaded job: {}", job_object.name);
                }
                job_objects.push(job_object);
            }
            Err(e) => {
                // Use the index `i` directly instead of searching again
                let job_path = &jobs_path
                    .get(i)
                    .and_then(|p| p.to_str())
                    .unwrap_or("unknown");
                if !quiet {
                    info!(
                        "Failed to parse job: \n Job path: {} \n Error: {}",
                        job_path.green(),
                        e.to_string().red()
                    );
                }
            }
        }
    }
    job_objects
}

pub fn get_jobs_paths() -> Vec<PathBuf> {
    let path = get_jobs_path();
    let mut jobs_path: Vec<PathBuf> = vec![];

    for job in fs::read_dir(path).unwrap() {
        jobs_path.push(job.unwrap().path());
    }
    jobs_path
}
