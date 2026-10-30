use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use autopilot_core_conditions::ConditionScheme;
use autopilot_core_task::TaskScheme;
use autopilot_core_time::When;

use super::TaskMode;

#[derive(Debug, Serialize, Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "AutoPilot job file")]
pub struct JobScheme {
    pub id: String,
    pub name: Option<String>,
    pub description: Option<String>,
    pub when: Option<When>,
    pub check_interval: Option<String>,
    pub conditions: Vec<ConditionScheme>,
    pub tasks: Vec<TaskScheme>,
    #[serde(default)]
    pub task_mode: TaskMode,
}
