use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
#[cfg(not(target_os = "linux"))]
use sysinfo::{ProcessRefreshKind, RefreshKind, System};

use crate::conditions::Condition;

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

/// Linux truncates `/proc/<pid>/comm` to 15 bytes, so a configured longer
/// name can still match when the observed name is the truncated prefix. Short
/// process names are only matched in the forward direction to avoid treating
/// e.g. `sshd` as a match for `ssh`.
#[cfg(target_os = "linux")]
fn process_name_matches(proc_name: &str, name: &str) -> bool {
    const LINUX_COMM_MAX_LEN: usize = 15;

    proc_name.contains(name)
        || (proc_name.len() == LINUX_COMM_MAX_LEN && name.starts_with(proc_name))
}

#[cfg(not(target_os = "linux"))]
fn process_name_matches(proc_name: &str, name: &str) -> bool {
    proc_name.contains(name)
}

#[cfg(target_os = "linux")]
fn is_process_running(name: &str) -> bool {
    use std::fs;

    let Ok(entries) = fs::read_dir("/proc") else {
        return false;
    };

    for entry in entries.flatten() {
        let file_name = entry.file_name();
        let Some(pid) = file_name.to_str().and_then(|s| s.parse::<u32>().ok()) else {
            continue;
        };
        let Ok(comm) = fs::read_to_string(format!("/proc/{pid}/comm")) else {
            continue;
        };
        let proc_name = comm.trim_end().to_lowercase();
        if process_name_matches(&proc_name, name) {
            return true;
        }
    }
    false
}

#[cfg(not(target_os = "linux"))]
fn is_process_running(name: &str) -> bool {
    let sys = System::new_with_specifics(
        RefreshKind::nothing().with_processes(ProcessRefreshKind::everything()),
    );

    sys.processes().values().any(|process| {
        let proc_name = process.name().to_string_lossy().to_lowercase();
        process_name_matches(&proc_name, name)
    })
}

impl Condition for ProcessCondition {
    fn check(&self) -> bool {
        let found = is_process_running(&self.process_name);

        if self.should_be_running {
            found
        } else {
            !found
        }
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "Process"
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Process condition")]
pub struct ProcessConditionScheme {
    #[serde(default)]
    #[schemars(
        title = "Process name",
        extend(
            "x-autopilot" = {
                "required": true,
                "widget": "dynamic-select",
                "provider": "processes",
                "allow_custom": true
            }
        )
    )]
    pub process_name: String,
    #[serde(default)]
    #[schemars(title = "Should be running")]
    pub should_be_running: Option<bool>,
}

#[cfg(test)]
#[path = "process_condition.spec.rs"]
mod spec;
