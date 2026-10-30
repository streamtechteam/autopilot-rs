use autopilot_core_conditions::Condition;
use autopilot_core_task::Task;
use autopilot_core_time::When;

mod conditions;
mod execution;
pub mod get;
pub mod repository;
pub mod schedule;
mod scheme;
pub mod set;
mod task_mode;

pub use execution::run_job;
pub use repository::{FsJobRepository, JobRepository};
pub use scheme::JobScheme;
pub use task_mode::{TaskMode, run_tasks};

#[derive(Clone)]
pub struct Job {
    pub id: String,
    pub name: String,
    pub description: String,
    pub when: Option<When>,
    //  TODO
    // i should probably make check_interval condition specific
    pub check_interval: Option<String>,
    pub conditions: Vec<Box<dyn Condition>>,
    pub tasks: Vec<Task>,
    pub task_mode: TaskMode,
    pub loaded: bool,
}

impl Job {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        id: String,
        name: String,
        description: String,
        check_interval: Option<String>,
        when: Option<When>,
        conditions: Vec<Box<dyn Condition>>,
        tasks: Vec<Task>,
        task_mode: TaskMode,
    ) -> Self {
        Job {
            id,
            name,
            description,
            when,
            check_interval,
            conditions,
            tasks,
            task_mode,
            loaded: false,
        }
    }

    pub fn from_scheme(scheme: JobScheme) -> Self {
        let conditions: Vec<Box<dyn Condition>> = scheme
            .conditions
            .into_iter()
            .map(|condition_scheme| condition_scheme.to_condition())
            .collect();

        let tasks: Vec<Task> = scheme
            .tasks
            .into_iter()
            .map(|task_scheme| Task::new(task_scheme.command))
            .collect();

        let when = scheme.when;
        Job {
            id: scheme.id.clone(),
            name: scheme.name.unwrap_or(format!("job_{}", scheme.id)),
            description: scheme.description.unwrap_or(" ".to_string()),
            when,
            check_interval: scheme.check_interval,
            conditions,
            tasks,
            task_mode: scheme.task_mode,
            loaded: false,
        }
    }

    pub fn from_json(json_str: &str) -> Result<Self, serde_json::Error> {
        let scheme: JobScheme = serde_json::from_str(json_str)?;
        Ok(Job::from_scheme(scheme))
    }

    pub fn add_condition(&mut self, condition: Box<dyn Condition>) {
        self.conditions.push(condition);
    }
}

#[cfg(test)]
#[path = "job.spec.rs"]
mod spec;

#[cfg(test)]
pub(crate) mod test_support;
