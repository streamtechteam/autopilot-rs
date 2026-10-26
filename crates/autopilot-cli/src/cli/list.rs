use std::io::stdout;

use colored::*;
use crossterm::{execute, terminal};
use dialoguer::{Confirm, Select, theme::ColorfulTheme};

use autopilot_core::{
    error::AutoPilotError,
    job::{FsJobRepository, JobRepository, JobScheme},
};
use autopilot_core_fs::paths;

pub async fn list() {
    match list_interactive().await {
        Ok(_) => {}
        Err(err) => eprintln!("Error: {}", err),
    }
}

fn format_job(job: &JobScheme) -> String {
    format!(
        "{} - {}",
        job.id,
        job.name.as_deref().unwrap_or(job.id.as_str())
    )
}

fn format_jobs(jobs: &[JobScheme]) -> Vec<String> {
    jobs.iter().map(format_job).collect()
}

pub async fn list_interactive() -> Result<(), AutoPilotError> {
    execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
    loop {
        let repository = FsJobRepository::new()?;
        let jobs = repository.list_jobs().await?;

        let formated_jobs = format_jobs(&jobs);

        if formated_jobs.is_empty() {
            println!("No jobs found");
            return Ok(());
        }
        let selected_job_index: usize = Select::with_theme(&ColorfulTheme::default())
            .with_prompt(format!(
                "Job (↑↓ nav, Enter action, ESC back) [root: {}]",
                paths()
                    .map(|paths| paths.root().display().to_string())
                    .unwrap_or_default()
            ))
            .items(&formated_jobs)
            .default(0)
            .interact()
            .map_err(|err| AutoPilotError::InvalidJob(err.to_string()))?;
        let selected_action = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select an action:")
            .items(["View Details", "Delete"])
            .default(0)
            .interact()
            .map_err(|err| AutoPilotError::InvalidJob(err.to_string()))?;
        let selected_job = &jobs[selected_job_index];
        match selected_action {
            0 => {
                execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
                println!(
                    "\nID : {}\nName: {}\n",
                    selected_job.id.yellow(),
                    selected_job
                        .name
                        .as_deref()
                        .unwrap_or(selected_job.id.as_str())
                        .green(),
                );
            }
            1 => {
                execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
                let confirm = Confirm::with_theme(&ColorfulTheme::default())
                    .with_prompt("Are you sure you want to delete this job?")
                    .interact()
                    .map_err(|err| AutoPilotError::InvalidJob(err.to_string()))?;
                if confirm {
                    let id = selected_job.id.clone();
                    execute!(stdout(), terminal::Clear(terminal::ClearType::All))?;
                    repository.delete_job(&id).await?;
                    println!("Deleting job {}", id);
                }
            }
            _ => unreachable!(),
        }
    }
}

#[cfg(test)]
#[path = "list.spec.rs"]
mod spec;
