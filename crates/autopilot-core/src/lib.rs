pub mod autopilot;
pub mod checks;
pub mod conditions;
pub mod cross_platform;
pub mod error;
pub mod fs;
pub mod job;
pub mod language;
pub mod logging;
pub mod status;
pub mod task;
pub mod time;
pub mod utilities;

// Root re-exports for the most commonly used types and functions.
pub use crate::{
    autopilot::AutoPilot,
    checks::{API_SERVICE_NAME, HealthResponse, check_if_running, port},
    conditions::{Condition, ConditionScheme},
    error::{AutoPilotError, Result},
    job::{
        Job,
        get::get_jobs,
        set::{add_job, remove_job},
    },
    status::{JobStatusEnum, JobStatusStruct, StateManager, StatusLog},
    task::Task,
    time::{DateTimeScheme, TimeScheme, When},
};
