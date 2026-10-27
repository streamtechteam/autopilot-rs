pub mod autopilot;
pub use autopilot_core_checks as checks;
pub use autopilot_core_conditions as conditions;
pub use autopilot_core_cross_platform as cross_platform;
pub use autopilot_core_error as error;
pub use autopilot_core_form as form;
pub use autopilot_core_fs as fs;
pub use autopilot_core_job as job;
pub use autopilot_core_language as language;
pub use autopilot_core_logging as logging;
pub use autopilot_core_status as status;
pub use autopilot_core_task as task;
pub use autopilot_core_utilities as utilities;

pub mod time {
    pub use autopilot_core_time::*;

    pub mod add {
        pub use autopilot_core_job::schedule::*;
    }
}

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
    time::{DateTimeScheme, MonthlyScheme, TimeScheme, WeeklyScheme, When, YearlyScheme},
};
