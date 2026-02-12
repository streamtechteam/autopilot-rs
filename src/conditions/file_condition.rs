use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};
use chrono::{DateTime, Duration, Local};
use dialoguer::{Input, Select, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;

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
        }
    }

    pub fn from_scheme(scheme: FileConditionScheme) -> Self {
        Self {
            path: scheme.path,
            check_type: scheme.check_type.to_lowercase(),
            time_threshold: scheme.time_threshold,
            size_threshold: scheme.size_threshold,
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

    fn create(&self) -> Result<ConditionScheme, AutoPilotError> {
        let path = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter file or directory path to monitor:")
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let check_types = ["File/Directory Exists", "Modified Recently", "Size Changed"];
        let selected_type = Select::with_theme(&ColorfulTheme::default())
            .with_prompt("Select check type:")
            .items(&check_types)
            .default(0)
            .interact_opt()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?
            .unwrap_or(0);

        let check_type = match selected_type {
            0 => "exists".to_string(),
            1 => "modified_recently".to_string(),
            2 => "size_changed".to_string(),
            _ => "exists".to_string(),
        };

        let (time_threshold, size_threshold) = match selected_type {
            1 => {
                // Modified recently
                let time_input: i64 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter time threshold in seconds (default: 300 for 5 minutes):")
                    .default(300)
                    .allow_empty(true)
                    .interact_text()
                    .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

                let time_threshold = if time_input == 0 {
                    Some(300)
                } else {
                    Some(time_input)
                };

                (time_threshold, None)
            }
            2 => {
                // Size changed
                let size_input: u64 = Input::with_theme(&ColorfulTheme::default())
                    .with_prompt("Enter size threshold in bytes:")
                    .interact_text()
                    .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

                let size_threshold = Some(size_input);

                (None, size_threshold)
            }
            _ => (None, None), // For "exists" check, no additional parameters needed
        };

        Ok(ConditionScheme::File(FileConditionScheme {
            path,
            check_type,
            time_threshold,
            size_threshold,
        }))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct FileConditionScheme {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub check_type: String,
    #[serde(default)]
    pub time_threshold: Option<i64>,
    #[serde(default)]
    pub size_threshold: Option<u64>,
}
