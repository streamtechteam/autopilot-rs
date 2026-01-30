use duct::cmd;
use serde::{Deserialize, Deserializer, Serialize};
use tokio_cron_scheduler::JobScheduler;

use crate::conditions::Condition;

#[derive(Clone)]
pub struct OutputCondition {
    command: String,
    target: String,
}
impl OutputCondition {
    pub fn new(command: String, target: String) -> Self {
        Self { command, target }
    }
    pub fn from_scheme(output_condition_scheme: OutputConditionScheme) -> Self {
        Self {
            command: output_condition_scheme.command,
            target: output_condition_scheme.target,
        }
    }
}
impl Condition for OutputCondition {
    fn check(&self) -> bool {
        sync_condition(&self.command, &self.target)
    }
}

pub fn sync_condition(command: &str, target: &str) -> bool {
    let output = cmd("sh", &["-c", &command])
        .read()
        .expect("Error while testing condition");
    output.trim() == target
}

pub async fn async_condition(command: &str, target: &str) -> bool {
    let output = cmd("sh", &["-c", &command])
        .read()
        .expect("Error while testing condition");
    output.trim() == target
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct OutputConditionScheme {
    pub command: String,
    pub target: String,
}
