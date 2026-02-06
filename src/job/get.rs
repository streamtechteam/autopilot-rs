use std::{fs, path::PathBuf};

use colored::*;
use log::{error, info};

use crate::{
    fs::get_jobs_path,
    job::{Job, JobScheme},
    utilities::jsonc_parser::jsonc_parse,
};

pub fn get_jobs(quiet: bool) -> Vec<Job> {
    let mut jobs_string: Vec<String> = vec![];
    let mut job_objects: Vec<Job> = vec![];
    let jobs_path = get_jobs_paths();
    for job in &jobs_path {
        match fs::read_to_string(job) {
            Ok(content) => jobs_string.push(content),
            Err(e) => {
                if !quiet {
                    if log::log_enabled!(log::Level::Error) {
                        error!("Failed to read job file {}: {}", job.display(), e);
                    } else {
                        eprintln!("Failed to read job file {}: {}", job.display(), e);
                    }
                }
            }
        }
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

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(job_entry) => jobs_path.push(job_entry.path()),
                    Err(e) => {
                        if log::log_enabled!(log::Level::Error) {
                            error!("Failed to read directory entry: {}", e)
                        } else {
                            eprintln!("Failed to read directory entry: {}", e)
                        }
                    }
                }
            }
        }
        Err(e) => {
            if log::log_enabled!(log::Level::Error) {
                error!("Failed to read jobs directory: {}", e);
            } else {
                eprintln!("Failed to read jobs directory: {}", e);
            }
        }
    }
    jobs_path
}
