use chrono::{NaiveDate, NaiveTime};
use colored::Colorize;
use dialoguer::Editor;
use dialoguer::{Confirm, Input, Select, theme::ColorfulTheme};
use std::path::PathBuf;

use autopilot_core::conditions::ConditionEntry;
use autopilot_core::cross_platform::get::get_supported_editors;
use autopilot_core::error::AutoPilotError;
use autopilot_core::job::{JobScheme, TaskMode};
use autopilot_core::task::TaskScheme;
use autopilot_core::time::{
    DateTimeScheme, MonthlyScheme, TimeScheme, WeeklyScheme, When, YearlyScheme,
};

use autopilot_core::job::{FsJobRepository, JobRepository};

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

fn prompt_schedule_number(
    prompt: &str,
    default: u8,
    min: u8,
    max: u8,
) -> Result<u8, AutoPilotError> {
    let input: String = Input::with_theme(&ColorfulTheme::default())
        .with_prompt(prompt)
        .default(default.to_string())
        .interact_text()
        .map_err(|err| {
            AutoPilotError::InvalidJob(format!("Failed to get schedule value: {err}"))
        })?;
    let value = input.trim().parse::<u8>().map_err(|_| {
        AutoPilotError::InvalidJob(format!(
            "Schedule value must be a number from {min} to {max}"
        ))
    })?;

    if !(min..=max).contains(&value) {
        return Err(AutoPilotError::InvalidJob(format!(
            "Schedule value must be between {min} and {max}, got {value}"
        )));
    }

    Ok(value)
}

fn prompt_task_mode() -> Result<TaskMode, AutoPilotError> {
    let modes = vec![
        "Concurrent (default)",
        "Sequential",
        "Sequential, stop after the first failure",
    ];
    let selected = Select::with_theme(&ColorfulTheme::default())
        .with_prompt("Choose how tasks should run:")
        .items(&modes)
        .default(0)
        .interact()
        .map_err(|err| AutoPilotError::InvalidJob(format!("Failed to select task mode: {err}")))?;

    Ok(match selected {
        0 => TaskMode::Concurrent,
        1 => TaskMode::Sequential,
        _ => TaskMode::SequentialStopOnFailure,
    })
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
                // Daily, Weekly, Monthly, Yearly
                let time_input: String = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter time of day (HH:MM):")
                    .interact_text()
                    .map_err(|err| {
                        AutoPilotError::InvalidJob(format!("Failed to get time: {}", err))
                    })?;

                let _time = NaiveTime::parse_from_str(&time_input, "%H:%M").map_err(|_| {
                    AutoPilotError::InvalidJob("Invalid time format. Please use HH:MM".to_string())
                })?;

                let time = time_input;

                match selected_index {
                    1 => Some(When::Daily(TimeScheme { time })),
                    2 => {
                        let weekday = prompt_schedule_number(
                            "Enter weekday (0/7 = Sunday, 1 = Monday, ..., 6 = Saturday):",
                            1,
                            0,
                            7,
                        )?;
                        Some(When::Weekly(WeeklyScheme { time, weekday }))
                    }
                    3 => {
                        let day = prompt_schedule_number("Enter day of month (1-31):", 1, 1, 31)?;
                        Some(When::Monthly(MonthlyScheme { time, day }))
                    }
                    4 => {
                        let month = prompt_schedule_number("Enter month (1-12):", 1, 1, 12)?;
                        let day = prompt_schedule_number("Enter day of month (1-31):", 1, 1, 31)?;
                        Some(When::Yearly(YearlyScheme { time, month, day }))
                    }
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
    let mut conditions: Vec<ConditionEntry> = Vec::new();
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
                conditions.push(ConditionEntry {
                    scheme: condition_scheme,
                    check_interval: None,
                });
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
        let command = if desired_editor == "Inline" {
            Input::with_theme(&ColorfulTheme::default())
                .with_prompt("Enter command to execute:")
                .interact_text()
                .map_err(|err| {
                    AutoPilotError::InvalidJob(format!("Failed to get command: {}", err))
                })?
        } else {
            Editor::new()
                .executable(desired_editor)
                .edit("")
                .map_err(|err| AutoPilotError::InvalidJob(err.to_string()))?
                .ok_or_else(|| AutoPilotError::Command("Command not provided".to_string()))?
        };
        tasks.push(TaskScheme { command });
    }

    let task_mode = prompt_task_mode()?;

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
        task_mode,
    };

    FsJobRepository::new()?.save_job(job).await
}

#[cfg(test)]
#[path = "create.spec.rs"]
mod spec;
