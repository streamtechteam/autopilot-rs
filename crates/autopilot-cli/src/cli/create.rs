use chrono::{NaiveDate, NaiveTime};
use colored::Colorize;
use dialoguer::Editor;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use std::path::PathBuf;

use autopilot_core::conditions::ConditionScheme;
use autopilot_core::cross_platform::get::get_supported_editors;
use autopilot_core::error::AutoPilotError;
use autopilot_core::job::JobScheme;
use autopilot_core::task::TaskScheme;
use autopilot_core::time::{DateTimeScheme, TimeScheme, When};

use autopilot_ui_api::{FileJobApi, JobApi};

use crate::cli::form::prompt_condition;

pub async fn create() {
    match create_interactive().await {
        Ok(job_file_path) => {
            println!("Job created successfully at: {}", job_file_path.display());
        }
        Err(e) => {
            eprintln!("Failed to create job: {}", e);
        }
    }
}

fn validate_cron_expression(cron_exp: &str) -> Result<When, AutoPilotError> {
    if cron_exp.split_whitespace().count() < 5 {
        return Err(AutoPilotError::InvalidJob(
            "Cron expression must have at least 5 fields (minute hour day-of-month month day-of-week)".to_string(),
        ));
    }

    Ok(When::Cron(cron_exp.to_string()))
}

async fn create_interactive() -> Result<PathBuf, AutoPilotError> {
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
        let schedule_types = vec![
            "Once (Specific Date and Time)",
            "Daily",
            "Weekly",
            "Monthly",
            "Yearly",
            "Cron Expression",
        ];

        let selected_index = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose a schedule type:")
            .items(&schedule_types)
            .default(0)
            .interact_opt()
            .map_err(|err| {
                AutoPilotError::InvalidJob(format!("Failed to select schedule type: {}", err))
            })?
            .ok_or_else(|| AutoPilotError::InvalidJob("No schedule type selected".to_string()))?;

        match selected_index {
            0 => {
                // Once
                let date_input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter date (YYYY/MM/DD):")
                    .default(
                        chrono::Local::now()
                            .date_naive()
                            .format("%Y/%m/%d")
                            .to_string(),
                    )
                    .interact_text()
                    .map_err(|err| {
                        AutoPilotError::InvalidJob(format!("Failed to get date: {}", err))
                    })?;

                let _date = NaiveDate::parse_from_str(&date_input, "%Y/%m/%d").map_err(|_| {
                    AutoPilotError::InvalidJob(
                        "Invalid date format. Please use YYYY/MM/DD".to_string(),
                    )
                })?;

                let time_input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter time (HH:MM):")
                    .interact_text()
                    .map_err(|err| {
                        AutoPilotError::InvalidJob(format!("Failed to get time: {}", err))
                    })?;

                let _time = NaiveTime::parse_from_str(&time_input, "%H:%M").map_err(|_| {
                    AutoPilotError::InvalidJob("Invalid time format. Please use HH:MM".to_string())
                })?;

                Some(When::Once(DateTimeScheme {
                    date: date_input,
                    time: time_input,
                }))
            }
            1..=4 => {
                // Daily, Weekly, Monthly, Yearly (all use TimeScheme)
                let time_input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter time of day (HH:MM):")
                    .interact_text()
                    .map_err(|err| {
                        AutoPilotError::InvalidJob(format!("Failed to get time: {}", err))
                    })?;

                let _time = NaiveTime::parse_from_str(&time_input, "%H:%M").map_err(|_| {
                    AutoPilotError::InvalidJob("Invalid time format. Please use HH:MM".to_string())
                })?;

                let time_scheme = TimeScheme { time: time_input };

                match selected_index {
                    1 => Some(When::Daily(time_scheme)),
                    2 => Some(When::Weekly(time_scheme)),
                    3 => Some(When::Monthly(time_scheme)),
                    4 => Some(When::Yearly(time_scheme)),
                    _ => unreachable!(),
                }
            }
            5 => {
                // Cron Expression
                let cron_exp: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter cron expression (e.g., 0 30 9 * * * for 9:30 AM daily):")
                    .interact_text()
                    .map_err(|err| {
                        AutoPilotError::InvalidJob(format!(
                            "Failed to get cron expression: {}",
                            err
                        ))
                    })?;

                Some(validate_cron_expression(&cron_exp)?)
            }
            _ => None,
        }
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

        match prompt_condition() {
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
            if tasks.is_empty() {
                println!("{}", "You must add at least one task.".red());
                continue;
            }
            break;
        }
        let mut supported_editors = get_supported_editors();
        supported_editors.insert(0, "Inline");
        let supported_editors = supported_editors;

        let desired_editor = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Choose your desired editor:")
            .default(0)
            .items(&supported_editors)
            .interact()
            .map_err(|err| AutoPilotError::InvalidJob(err.to_string()))?;

        let desired_editor = supported_editors[desired_editor];
        let command;
        if desired_editor == "Inline" {
            command = Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter command to execute:")
                .interact_text()
                .map_err(|err| {
                    AutoPilotError::InvalidJob(format!("Failed to get command: {}", err))
                })?;
        } else {
            command = Editor::new()
                .executable(desired_editor)
                .edit("")
                .map_err(|err| AutoPilotError::InvalidJob(err.to_string()))?
                .ok_or_else(|| AutoPilotError::Command("Command not provided".to_string()))?;
        }
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

    let job = JobScheme {
        id: format!("job_{}", chrono::Utc::now().timestamp_millis()),
        name: Some(name),
        description: Some(description),
        when,
        check_interval,
        conditions,
        tasks,
    };

    FileJobApi::new()?.save_job(job).await
}

#[cfg(test)]
#[path = "create.spec.rs"]
mod spec;
