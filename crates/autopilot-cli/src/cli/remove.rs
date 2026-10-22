use colored::Colorize;
use dialoguer::{Confirm, Select, theme::ColorfulTheme};

use autopilot_core::{error::AutoPilotError, fs::get_autopilot_path, job::JobScheme};
use autopilot_ui_api::{FileJobApi, JobApi};

pub async fn remove() {
    match remove_interactive().await {
        Ok(job_id) => {
            println!("Job removed successfully: {}", job_id);
        }
        Err(e) => {
            eprintln!("Failed to remove job\n{}", e);
        }
    }
}

fn format_job_option(job: &JobScheme) -> String {
    format!("{} - {}", job.name.as_deref().unwrap_or("Unknown"), job.id)
}

pub async fn remove_interactive() -> Result<String, AutoPilotError> {
    println!(
        "{}",
        "This subcommand is now deprecated\nPlease use autopilot list instead".red()
    );
    let api = FileJobApi::new()?;
    let jobs = api.list_jobs().await?;
    let options: Vec<String> = jobs.iter().map(format_job_option).collect();

    if options.is_empty() {
        return Err(AutoPilotError::Job("No jobs exist to remove!".to_string()));
    }

    let selection = loop {
        let selection = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Select an option to remove (root: {})",
                get_autopilot_path()
            ))
            .default(0)
            .items(&options)
            .interact()
            .map_err(|err| AutoPilotError::InvalidJob(err.to_string()))?;
        let confirm = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Are you sure you want to remove {} ?",
                options[selection]
            ))
            .default(false)
            .interact()
            .map_err(|err| AutoPilotError::InvalidJob(err.to_string()))?;

        if confirm {
            break selection;
        }
    };

    let job = &jobs[selection];
    println!("Removing job");
    if !api.delete_job(&job.id).await? {
        return Err(AutoPilotError::Job(format!(
            "Job '{}' no longer exists",
            job.id
        )));
    }

    Ok(job.id.clone())
}

#[cfg(test)]
#[path = "remove.spec.rs"]
mod spec;
