use crate::conditions::Condition;
use serde::{Deserialize, Serialize};
use sysinfo::System;

/// Represents a process monitor condition
#[derive(Clone)]
pub struct ProcessCondition {
    /// Name of the process to check for
    pub process_name: String,
    /// Whether the process should be running (true) or not running (false)
    pub should_be_running: bool,
}

impl ProcessCondition {
    pub fn new(process_name: String, should_be_running: bool) -> Self {
        Self {
            process_name: process_name.to_lowercase(),
            should_be_running,
        }
    }

    pub fn from_scheme(scheme: ProcessConditionScheme) -> Self {
        Self {
            process_name: scheme.process_name.to_lowercase(),
            should_be_running: scheme.should_be_running.unwrap_or(true),
        }
    }
}

impl Condition for ProcessCondition {
    fn check(&self) -> bool {
        let mut sys = System::new_all();
        sys.refresh_all();

        let mut found = false;
        for process in sys.processes().values() {
            let proc_name = process.name().to_string_lossy().to_lowercase();
            if proc_name.contains(&self.process_name) || self.process_name.contains(&proc_name) {
                found = true;
                break;
            }
        }

        if self.should_be_running {
            found
        } else {
            !found
        }
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct ProcessConditionScheme {
    pub process_name: String,
    pub should_be_running: Option<bool>,
}
