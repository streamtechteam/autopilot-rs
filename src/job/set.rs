use std::path::PathBuf;

use serde_json::json;

use crate::{
    conditions::ConditionScheme,
    cron::DateTimeScheme,
    error::AutoPilotError,
    fs::{self, get_jobs_dir},
    job::JobScheme,
    task::TaskScheme,
};

pub fn set_job(
    name: Option<String>,
    description: Option<String>,
    when: Option<DateTimeScheme>,
    check_interval: Option<String>,
    conditions: Vec<ConditionScheme>,
    tasks: Vec<TaskScheme>,
) -> Result<PathBuf, AutoPilotError> {
    // Generate unique ID
    let id = format!("job_{}", chrono::Utc::now().timestamp());

    let job = JobScheme {
        id: id.clone(),
        name: name.clone(),
        description,
        when,
        check_interval,
        conditions,
        tasks,
    };

    // Write to file
    let jobs_dir = get_jobs_dir()?;
    let job_file_path = jobs_dir.join(format!(
        "{}.jsonc",
        if name.is_some() {
            format!("{}_{}", id, name.unwrap())
        } else {
            id
        }
    ));

    fs::set_jobs_path().map_err(|e| {
        AutoPilotError::InvalidJob(format!("Failed to create jobs directory: {}", e))
    })?;

    std::fs::write(&job_file_path, serde_json::to_string_pretty(&job).unwrap())
        .map_err(|e| AutoPilotError::InvalidJob(format!("Failed to write job file: {}", e)))?;

    Ok(job_file_path)
}
