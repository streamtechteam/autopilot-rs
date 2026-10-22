use std::{
    collections::HashSet,
    fs::{self},
    path::{Path, PathBuf},
    process::exit,
};

use colored::*;
use log::{error, info};

use crate::{
    error::AutoPilotError,
    fs::get_jobs_path,
    job::{Job, JobScheme},
    utilities::{directory_search::search_directory, jsonc_parser::jsonc_parse},
};

pub fn get_jobs(quiet: bool) -> Vec<Job> {
    let mut job_files: Vec<(PathBuf, String)> = vec![];
    let mut job_objects: Vec<Job> = vec![];
    let mut jobs_ids: Vec<String> = vec![];
    let jobs_path = get_jobs_paths();
    for job in &jobs_path {
        match fs::read_to_string(job) {
            Ok(content) => job_files.push((job.clone(), content)),
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

    for (job_path, job_str) in &job_files {
        match parse_job(job_str) {
            Ok(job_scheme) => {
                let job_object = Job::from_scheme(job_scheme);
                let id = job_object.id.clone();
                if !quiet {
                    info!("Loaded job: {}", job_object.name);
                }
                jobs_ids.push(id);

                job_objects.push(job_object);
            }
            Err(e) => {
                if !quiet {
                    info!(
                        "Failed to parse job: \n Job path: {} \n Error: {}",
                        job_path.display().to_string().green(),
                        e.to_string().red()
                    );
                }
            }
        }
    }

    log_all_duplicates(&jobs_ids);
    job_objects
}

fn parse_job(content: &str) -> Result<JobScheme, serde_json::Error> {
    match serde_json::from_str(content) {
        Ok(scheme) => Ok(scheme),
        Err(_) => serde_json::from_str(&jsonc_parse(content)),
    }
}

fn log_all_duplicates(jobs_ids: &[String]) {
    let mut seen = HashSet::new();
    let mut dupes = HashSet::new();

    for id in jobs_ids {
        if !seen.insert(id.clone()) {
            // id was already seen → it’s a duplicate
            if dupes.insert(id.clone()) {
                // first time we notice this specific duplicate
                eprintln!(
                    "{} {}",
                    "[WARNING] Duplicate job id found: ".yellow(),
                    id.red()
                );
                search_directory(Path::new(get_jobs_path().as_str()), id).unwrap()
            }
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
    let path = get_jobs_path();
    let mut jobs_path: Vec<PathBuf> = vec![];

    match fs::read_dir(path) {
        Ok(entries) => {
            for entry in entries {
                match entry {
                    Ok(job_entry) => {
                        if job_entry.path().is_file() {
                            jobs_path.push(job_entry.path());
                        }
                    }
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

pub fn get_job(path: PathBuf) -> Result<Job, AutoPilotError> {
    match fs::read_to_string(&path) {
        Ok(content) => match parse_job(&content) {
            Ok(job_scheme) => Ok(Job::from_scheme(job_scheme)),
            Err(e) => Err(AutoPilotError::InvalidJob(format!(
                "Failed to parse job file {}: {}",
                path.display(),
                e
            )))?,
        },
        Err(e) => Err(AutoPilotError::Json(format!(
            "Failed to read job file {}: {}",
            path.display(),
            e
        )))?,
    }
}

#[cfg(test)]
#[path = "get.spec.rs"]
mod spec;
