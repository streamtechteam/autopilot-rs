use std::{collections::HashSet, path::PathBuf};

use colored::*;
use log::{error, info};

use autopilot_core_fs::{JsonFileStore, paths};

use autopilot_core_error::AutoPilotError;

use crate::job::{Job, JobScheme};

/// Loads all jobs, returning an error when the storage cannot be read or the
/// store contains duplicate job ids.
pub fn get_jobs(quiet: bool) -> Result<Vec<Job>, AutoPilotError> {
    let store = JsonFileStore::<JobScheme>::new(paths()?.jobs_dir());
    let records = store.list()?;

    let mut job_objects: Vec<Job> = Vec::new();
    let mut jobs_ids: Vec<String> = Vec::new();

    for (_, job_scheme) in records {
        let job_object = Job::from_scheme(job_scheme);
        let id = job_object.id.clone();
        if !quiet {
            info!("Loaded job: {}", job_object.name);
        }
        jobs_ids.push(id);
        job_objects.push(job_object);
    }

    let duplicates = duplicate_ids(&jobs_ids);
    if !duplicates.is_empty() {
        for id in &duplicates {
            eprintln!(
                "{} {}",
                "[CRITICAL] Duplicate job id found: ".yellow(),
                id.red()
            );
        }
        return Err(AutoPilotError::InvalidJob(format!(
            "Duplicate job ids must be resolved before running autopilot: {}",
            duplicates.join(", ")
        )));
    }

    Ok(job_objects)
}

#[cfg(test)]
fn parse_job(content: &str) -> Result<JobScheme, serde_json::Error> {
    serde_json::from_str(content)
        .or_else(|_| serde_json::from_str(&autopilot_core_fs::jsonc_parse(content)))
}

fn duplicate_ids(jobs_ids: &[String]) -> Vec<String> {
    let mut seen = HashSet::new();
    let mut duplicates = Vec::new();

    for id in jobs_ids {
        if !seen.insert(id.clone()) && !duplicates.contains(id) {
            duplicates.push(id.clone());
        }
    }

    duplicates.sort();
    duplicates
}

pub fn get_jobs_paths() -> Vec<PathBuf> {
    let Ok(paths) = paths() else {
        return Vec::new();
    };
    let store = JsonFileStore::<JobScheme>::new(paths.jobs_dir());

    match store.file_paths() {
        Ok(paths) => paths,
        Err(error) => {
            if log::log_enabled!(log::Level::Error) {
                error!("Failed to read jobs directory: {}", error);
            } else {
                eprintln!("Failed to read jobs directory: {}", error);
            }
            Vec::new()
        }
    }
}

pub fn get_job(path: PathBuf) -> Result<Job, AutoPilotError> {
    let store = JsonFileStore::<JobScheme>::new(paths()?.jobs_dir());
    let scheme = store.read_path(&path)?;
    Ok(Job::from_scheme(scheme))
}

#[cfg(test)]
#[path = "get.spec.rs"]
mod spec;
