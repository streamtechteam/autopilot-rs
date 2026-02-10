use std::path::PathBuf;

use colored::Colorize;
use dialoguer::{Confirm, Error, Select, theme::ColorfulTheme};

use crate::{
    error::AutoPilotError,
    fs::get_autopilot_path,
    job::{
        get::{get_job, get_jobs, get_jobs_paths},
        set::remove_job,
    },
};

pub fn remove() {
    match remove_interactive() {
        Ok(job_file_path) => {
            println!("Job removed successfully at: {}", job_file_path.display());
        }
        Err(e) => {
            eprintln!("Failed to remove job\n{}", e);
        }
    }
}

pub fn remove_interactive() -> Result<PathBuf, AutoPilotError> {
    println!(
        "{}",
        "This subcommand is now deprecated\nPlease use autopilot list instead".red()
    );
    let job_paths = get_jobs_paths();
    let options: Vec<String> = job_paths
        .iter()
        .map(|value| {
            // println!("Processing job: {}", value.display());
            let job_name = match get_job(value.clone()) {
                Ok(job) => Some(job.name),
                Err(_) => None,
            };

            format!(
                "{} - {}",
                job_name.unwrap_or("Unknown".to_string()),
                value
                    .file_name()
                    .expect("Failed to get file name")
                    .to_str()
                    .expect("Failed to convert filename to String (filename may have unsupported characters)")
            )
        })
        .collect();
    let mut selection;
    let mut confirm;
    if options.is_empty() {
        // eprintln!("No jobs found");

        return Err(AutoPilotError::Job("No jobs exist to remove!".to_string()));
    }
    loop {
        selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Select an option to remove (root: {})",
                get_autopilot_path()
            ))
            .default(0)
            .items(&options)
            .interact()
            .map_err(|err| AutoPilotError::Dialoguer(err))?;
        confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Are you sure you want to remove {} ?",
                options[selection]
            ))
            .default(false)
            .interact()
            .map_err(|err| AutoPilotError::Dialoguer(err))?;
        if confirm {
            break;
        }
    }
    if confirm {
        println!("Removing job");
        remove_job(
            None,
            Some(
                job_paths[selection]
                    .clone()
                    .to_str()
                    .expect("Failed to convert path to string")
                    .to_string(),
            ),
        )?;
    }
    Ok(job_paths[selection].clone())
}
