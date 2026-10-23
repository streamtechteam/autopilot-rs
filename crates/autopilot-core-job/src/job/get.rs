use std::{
    collections::HashSet,
    path::{Path, PathBuf},
    process::exit,
};

use colored::*;
use log::{error, info};

use autopilot_core_fs::{JsonFileStore, paths};

use autopilot_core_error::AutoPilotError;
use autopilot_core_utilities::directory_search::search_directory;

use crate::job::{Job, JobScheme};

pub fn get_jobs(quiet: bool) -> Vec<Job> {
    let Ok(paths) = paths() else {
        log_storage_error(quiet, "Filesystem paths have not been initialized");
        return Vec::new();
    };
    let jobs_dir = paths.jobs_dir();
    let store = JsonFileStore::<JobScheme>::new(&jobs_dir);
    let records = match store.list() {
        Ok(records) => records,
        Err(error) => {
            log_storage_error(quiet, &format!("Failed to read jobs directory: {error}"));
            return Vec::new();
        }
    };

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

    log_all_duplicates(&jobs_ids, &jobs_dir);
    job_objects
}

fn log_storage_error(quiet: bool, message: &str) {
    if !quiet {
        if log::log_enabled!(log::Level::Error) {
            error!("{message}");
        } else {
            eprintln!("{message}");
        }
    }
}

#[cfg(test)]
fn parse_job(content: &str) -> Result<JobScheme, serde_json::Error> {
    serde_json::from_str(content)
        .or_else(|_| serde_json::from_str(&autopilot_core_fs::jsonc_parse(content)))
}

fn log_all_duplicates(jobs_ids: &[String], jobs_path: &Path) {
    let mut seen = HashSet::new();
    let mut dupes = HashSet::new();

    for id in jobs_ids {
        if !seen.insert(id.clone()) && dupes.insert(id.clone()) {
            eprintln!(
                "{} {}",
                "[WARNING] Duplicate job id found: ".yellow(),
                id.red()
            );
            search_directory(jobs_path, id).unwrap()
        }
    }

    if !dupes.is_empty() {
        eprintln!(
            "{}",
            "[CRITICAL] Duplicates should be resolved before running autopilot".red()
        );

        exit(1);
    }
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
