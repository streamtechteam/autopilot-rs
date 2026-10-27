use chrono::{DateTime, Duration, Local};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

use crate::conditions::Condition;

/// Represents a file/path monitor condition
#[derive(Clone)]
pub struct FileCondition {
    /// Path to the file or directory to monitor
    pub path: String,
    /// Type of check: "exists", "modified_recently", "size_changed"
    pub check_type: String,
    /// Time threshold for "modified_recently" in seconds (default: 300 for 5 minutes)
    pub time_threshold: Option<i64>,
    /// Size threshold for "size_changed" in bytes
    pub size_threshold: Option<u64>,
    /// Per-condition polling interval override in milliseconds.
    /// `None` means the job-level `check_interval` applies.
    pub check_interval: Option<String>,
}

impl FileCondition {
    pub fn new(
        path: String,
        check_type: String,
        time_threshold: Option<i64>,
        size_threshold: Option<u64>,
    ) -> Self {
        Self {
            path,
            check_type: check_type.to_lowercase(),
            time_threshold,
            size_threshold,
            check_interval: None,
        }
    }

    pub fn from_scheme(scheme: FileConditionScheme) -> Self {
        Self {
            path: scheme.path,
            check_type: scheme.check_type.to_lowercase(),
            time_threshold: scheme.time_threshold,
            size_threshold: scheme.size_threshold,
            check_interval: scheme.check_interval,
        }
    }
}

impl Condition for FileCondition {
    fn check(&self) -> bool {
        let path = Path::new(&self.path);

        match self.check_type.as_str() {
            "exists" => path.exists(),

            "modified_recently" => {
                if !path.exists() {
                    return false;
                }

                let metadata = match fs::metadata(path) {
                    Ok(metadata) => metadata,
                    Err(_) => return false,
                };

                let modified_time = match metadata.modified() {
                    Ok(time) => time,
                    Err(_) => return false,
                };

                let modified_local: DateTime<Local> = modified_time.into();
                let now = Local::now();
                let threshold_seconds = self.time_threshold.unwrap_or(300);
                let threshold_duration = Duration::seconds(threshold_seconds);

                now.signed_duration_since(modified_local) < threshold_duration
            }

            "size_changed" => {
                if !path.exists() {
                    return false;
                }

                let metadata = match fs::metadata(path) {
                    Ok(metadata) => metadata,
                    Err(_) => return false,
                };

                let current_size = metadata.len();
                let threshold_size = self.size_threshold.unwrap_or(0);

                current_size >= threshold_size
            }

            _ => false,
        }
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "File"
    }
    fn check_interval(&self) -> Option<&str> {
        self.check_interval.as_deref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "File condition")]
pub struct FileConditionScheme {
    #[serde(default)]
    #[schemars(
        title = "Path",
        extend("x-autopilot" = {"required": true})
    )]
    pub path: String,
    #[serde(default)]
    #[schemars(
        title = "Check type",
        extend(
            "x-autopilot" = {
                "required": true,
                "widget": "select",
                "options": [
                    {"label": "File/Directory Exists", "value": "exists"},
                    {"label": "Modified Recently", "value": "modified_recently"},
                    {"label": "Size Changed", "value": "size_changed"}
                ]
            }
        )
    )]
    pub check_type: String,
    #[serde(default)]
    #[schemars(
        title = "Time threshold",
        range(min = 0),
        extend("x-autopilot" = {"unit": "seconds"})
    )]
    pub time_threshold: Option<i64>,
    #[serde(default)]
    #[schemars(
        title = "Size threshold",
        range(min = 0),
        extend("x-autopilot" = {"unit": "bytes"})
    )]
    pub size_threshold: Option<u64>,
    #[serde(default)]
    #[schemars(
        title = "Check interval",
        description = "Milliseconds between checks. Overrides the job-level check_interval."
    )]
    pub check_interval: Option<String>,
}

#[cfg(test)]
#[path = "file_condition.spec.rs"]
mod spec;
