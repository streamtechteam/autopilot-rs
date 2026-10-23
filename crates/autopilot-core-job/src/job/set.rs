use std::path::PathBuf;

use autopilot_core_fs::{JsonFileStore, paths};

use autopilot_core_conditions::ConditionScheme;
use autopilot_core_error::AutoPilotError;
use autopilot_core_task::TaskScheme;
use autopilot_core_time::When;

use crate::job::JobScheme;

pub fn add_job(
    name: Option<String>,
    description: Option<String>,
    when: Option<When>,
    check_interval: Option<String>,
    conditions: Vec<ConditionScheme>,
    tasks: Vec<TaskScheme>,
) -> Result<PathBuf, AutoPilotError> {
    let id = format!("job_{}", chrono::Utc::now().timestamp());

    let job = JobScheme {
        id,
        name,
        description,
        when,
        check_interval,
        conditions,
        tasks,
    };

    let store = JsonFileStore::<JobScheme>::new(paths()?.jobs_dir());
    Ok(store.save(&job)?)
}

pub fn remove_job(id: Option<String>, file_name: Option<String>) -> Result<(), AutoPilotError> {
    if id.is_none() && file_name.is_none() {
        return Err(AutoPilotError::InvalidJob(
            "No job ID or file name provided".to_string(),
        ));
    }

    let store = JsonFileStore::<JobScheme>::new(paths()?.jobs_dir());

    if let Some(file_name) = file_name {
        let path = if file_name.ends_with(".jsonc") {
            PathBuf::from(file_name)
        } else {
            PathBuf::from(format!("{file_name}.jsonc"))
        };
        store.remove_path(path)?;
    }

    if let Some(id) = id
        && !store.delete(&id)?
    {
        return Err(AutoPilotError::Job(
            "Job associated with provided job id not found".to_string(),
        ));
    }

    Ok(())
}

#[cfg(test)]
#[path = "set.spec.rs"]
mod spec;
