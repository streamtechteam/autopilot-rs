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
    /// Type of check: "exists", "modified_recently", "min_size"
    pub check_type: String,
    /// Time threshold for "modified_recently" in seconds (default: 300 for 5 minutes)
    pub time_threshold: Option<i64>,
    /// Minimum size for "min_size" in bytes (default: 0)
    pub size_threshold: Option<u64>,
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
            check_type: normalize_check_type(&check_type),
            time_threshold,
            size_threshold,
        }
    }

    pub fn from_scheme(scheme: FileConditionScheme) -> Self {
        Self {
            path: scheme.path,
            check_type: normalize_check_type(&scheme.check_type),
            time_threshold: scheme.time_threshold,
            size_threshold: scheme.size_threshold,
        }
    }
}

/// Canonical check type, mapping the old `size_changed` name to `min_size`.
fn normalize_check_type(check_type: &str) -> String {
    match check_type.to_lowercase().as_str() {
        "size_changed" => "min_size".to_string(),
        other => other.to_string(),
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

            "min_size" | "size_changed" => {
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
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "File condition")]
pub struct FileConditionScheme {
    #[serde(default)]
    #[schemars(
        title = "Path",
        extend("x-autopilot" = {"required": true, "group": "file"})
    )]
    pub path: String,
    /// Check to run: `exists`, `modified_recently`, or `min_size`.
    ///
    /// `size_changed` is still accepted as a deprecated alias for `min_size`.
    #[serde(default)]
    #[schemars(
        title = "Check type",
        extend(
            "x-autopilot" = {
                "required": true,
                "widget": "select",
                "group": "check",
                "options": [
                    {"label": "File/Directory Exists", "value": "exists"},
                    {"label": "Modified Recently", "value": "modified_recently"},
                    {"label": "Minimum Size", "value": "min_size"}
                ]
            }
        )
    )]
    pub check_type: String,
    #[serde(default)]
    #[schemars(
        title = "Time threshold",
        range(min = 0),
        extend(
            "x-autopilot" = {
                "unit": "seconds",
                "group": "check",
                "visible_when": {"field": "check_type", "equals": "modified_recently"}
            }
        )
    )]
    pub time_threshold: Option<i64>,
    #[serde(default)]
    #[schemars(
        title = "Minimum size",
        range(min = 0),
        extend(
            "x-autopilot" = {
                "unit": "bytes",
                "group": "check",
                "visible_when": {"field": "check_type", "equals": "min_size"}
            }
        )
    )]
    pub size_threshold: Option<u64>,
}

#[cfg(test)]
#[path = "file_condition.spec.rs"]
mod spec;
