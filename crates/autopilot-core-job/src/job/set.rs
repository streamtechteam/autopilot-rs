use std::{
    path::{Path, PathBuf},
    sync::atomic::{AtomicU64, Ordering},
    time::{SystemTime, UNIX_EPOCH},
};

use autopilot_core_fs::config;
use autopilot_core_fs::storage::JsonFileStore;

use autopilot_core_conditions::ConditionScheme;
use autopilot_core_error::AutoPilotError;
use autopilot_core_task::TaskScheme;
use autopilot_core_time::When;

use crate::job::{JobScheme, TaskMode};

pub fn add_job(
    name: Option<String>,
    description: Option<String>,
    when: Option<When>,
    check_interval: Option<String>,
    conditions: Vec<ConditionScheme>,
    tasks: Vec<TaskScheme>,
    task_mode: TaskMode,
) -> Result<PathBuf, AutoPilotError> {
    let id = next_job_id();

    let job = JobScheme {
        id,
        name,
        description,
        when,
        check_interval,
        conditions,
        tasks,
        task_mode,
    };

    let store = JsonFileStore::<JobScheme>::new(config::current()?.jobs_dir());
    Ok(store.save(&job)?)
}

fn next_job_id() -> String {
    static COUNTER: AtomicU64 = AtomicU64::new(0);

    let nanosecond = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_nanos())
        .unwrap_or_default();
    let sequence = COUNTER.fetch_add(1, Ordering::Relaxed);
    format!("job_{nanosecond}_{sequence}")
}

#[allow(clippy::too_many_arguments)]
pub fn update_job(
    id: &str,
    name: Option<String>,
    description: Option<String>,
    when: Option<When>,
    check_interval: Option<String>,
    conditions: Vec<ConditionScheme>,
    tasks: Vec<TaskScheme>,
    task_mode: Option<TaskMode>,
) -> Result<PathBuf, AutoPilotError> {
    let store = JsonFileStore::<JobScheme>::new(config::current()?.jobs_dir());

    let Some((path, mut job)) = store.list()?.into_iter().find(|(_, job)| job.id == id) else {
        return Err(AutoPilotError::Job(
            "Job associated with provided job id not found".to_string(),
        ));
    };

    job.name = name.or(job.name);
    job.description = description.or(job.description);
    job.when = when;
    job.check_interval = check_interval;
    job.conditions = conditions;
    job.tasks = tasks;
    if let Some(task_mode) = task_mode {
        job.task_mode = task_mode;
    }

    store.write_path(&path, &job)?;
    Ok(path)
}

pub fn remove_job(id: Option<String>, file_name: Option<String>) -> Result<(), AutoPilotError> {
    if id.is_none() && file_name.is_none() {
        return Err(AutoPilotError::InvalidJob(
            "No job ID or file name provided".to_string(),
        ));
    }

    let store = JsonFileStore::<JobScheme>::new(config::current()?.jobs_dir());

    if let Some(file_name) = file_name {
        let candidate = Path::new(&file_name);
        let Some(name) = candidate.file_name().and_then(|name| name.to_str()) else {
            return Err(AutoPilotError::InvalidJob(
                "Job file name must be a single file name".to_string(),
            ));
        };
        if candidate.components().count() != 1 || name.is_empty() || name == "." || name == ".." {
            return Err(AutoPilotError::InvalidJob(
                "Job file name must not contain path separators".to_string(),
            ));
        }

        let file_name = if name.ends_with(".jsonc") {
            name.to_string()
        } else {
            format!("{name}.jsonc")
        };
        store.remove_path(store.dir().join(file_name))?;
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
