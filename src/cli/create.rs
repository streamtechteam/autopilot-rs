use chrono::{NaiveDate, NaiveTime};
use colored::Colorize;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use log::error;
use serde_json::json;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use strum::IntoEnumIterator;

use crate::conditions::{Condition, ConditionScheme};
use crate::cron::DateTimeScheme;
use crate::error::AutoPilotError;
use crate::fs::get_jobs_dir;
use crate::job::JobScheme;
use crate::job::set::add_job;
use crate::task::TaskScheme;

pub fn create() {
    match create_interactive() {
        Ok(job_file_path) => {
            println!("Job created successfully at: {}", job_file_path.display());
        }
        Err(e) => {
            eprintln!("Failed to create job: {}", e);
        }
    }
}

fn create_interactive() -> Result<PathBuf, AutoPilotError> {
    // Get job basic information
    let name: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter job name:")
        .interact_text()
        .map_err(|err| AutoPilotError::InvalidJob(format!("Failed to get job name: {}", err)))?;

    let description: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt("Enter job description (optional):")
        .allow_empty(true)
        .interact_text()
        .map_err(|err| {
            AutoPilotError::InvalidJob(format!("Failed to get job description: {}", err))
        })?;

    // Handle time-based execution
    let when = if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to schedule this job for a specific time?")
        .interact_opt()
        .map_err(|err| {
            AutoPilotError::InvalidJob(format!("Failed to get time preference: {}", err))
        })?
        .unwrap_or(false)
    {
        let date_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter date (YYYY/MM/DD):")
            .interact_text()
            .map_err(|err| AutoPilotError::InvalidJob(format!("Failed to get date: {}", err)))?;

        let date = NaiveDate::parse_from_str(&date_input, "%Y/%m/%d").map_err(|_| {
            AutoPilotError::InvalidJob("Invalid date format. Please use YYYY/MM/DD".to_string())
        })?;

        let time_input: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter time (HH:MM:SS):")
            .with_initial_text("00:00:00".to_string())
            .interact_text()
            .map_err(|err| AutoPilotError::InvalidJob(format!("Failed to get time: {}", err)))?;

        let time = NaiveTime::parse_from_str(&time_input, "%H:%M:%S").map_err(|_| {
            AutoPilotError::InvalidJob("Invalid time format. Please use HH:MM:SS".to_string())
        })?;

        // Some(json!({
        //     "date": date.format("%Y/%m/%d").to_string(),
        //     "time": time.format("%H:%M:%S").to_string()
        // }))
        Some(DateTimeScheme {
            date: date.format("%Y/%m/%d").to_string(),
            time: time.format("%H:%M:%S").to_string(),
        })
    } else {
        None
    };

    // Collect conditions
    let mut conditions: Vec<ConditionScheme> = Vec::new();
    loop {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to add a condition?")
            .interact_opt()
            .map_err(|err| {
                AutoPilotError::InvalidJob(format!("Failed to get condition preference: {}", err))
            })?
            .unwrap_or(false)
        {
            break;
        }

        // Get available condition types
        let condition_names: Vec<String> = ConditionScheme::varient_names();

        let selected_index = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose a condition type:")
            .items(
                &condition_names
                    .iter()
                    .map(|s| s.as_str())
                    .collect::<Vec<_>>(),
            )
            .default(0)
            .interact_opt()
            .map_err(|err| {
                AutoPilotError::InvalidJob(format!("Failed to select condition type: {}", err))
            })?
            .ok_or_else(|| AutoPilotError::InvalidJob("No condition type selected".to_string()))?;

        let selected_condition = ConditionScheme::iter()
            .nth(selected_index)
            .expect("Error happened when creating condition")
            .to_condition();
        match selected_condition.create() {
            Ok(condition_scheme) => {
                conditions.push(condition_scheme);
                println!("Condition added successfully!");
            }
            Err(e) => {
                eprintln!("Failed to create condition: {}", e);
                continue;
            }
        }
    }

    // Collect tasks
    let mut tasks: Vec<TaskScheme> = Vec::new();
    loop {
        if !Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Do you want to add a task? (at least one is required)")
            .interact_opt()
            .map_err(|err| {
                AutoPilotError::InvalidJob(format!("Failed to get task preference: {}", err))
            })?
            .unwrap_or(false)
        {
            if tasks.len() == 0 {
                println!("{}", "You must add at least one task.".red());
                continue;
            }
            break;
        }

        let command: String = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter command to execute:")
            .interact_text()
            .map_err(|err| AutoPilotError::InvalidJob(format!("Failed to get command: {}", err)))?;

        tasks.push(TaskScheme { command });
    }

    let check_interval = if Confirm::with_theme(&ColorfulTheme::default())
        .with_prompt("Do you want to add a check interval (autopilot will only check once on startup by default)?")
        .interact_opt()
        .map_err(|err| {
            AutoPilotError::InvalidJob(format!("Failed to get check_interval preference: {}", err))
        })?
        .unwrap_or(false)
    {
        let check_interval = Input::<String>::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter check interval in milliseconds:")
            .interact_text()
            .map_err(|err| {
                AutoPilotError::InvalidJob(format!("Failed to get check interval: {}", err))
            })?;

        Some(check_interval)
    }else {
        None
    };
    let job_file_path = add_job(
        Some(name),
        Some(description),
        when,
        check_interval,
        conditions,
        tasks,
    );
    job_file_path
}
