use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};
use dialoguer::{Input, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use sysinfo::{Disks, System};

/// Represents a disk space condition
#[derive(Clone)]
pub struct DiskSpaceCondition {
    /// Path to the disk/mount point to check (e.g., "/", "C:\\", "/home")
    pub path: String,
    /// Minimum free space required in GB
    pub min_free_gb: f64,
    /// Maximum used space allowed in GB
    pub max_used_gb: Option<f64>,
}

impl DiskSpaceCondition {
    pub fn new(path: String, min_free_gb: f64, max_used_gb: Option<f64>) -> Self {
        Self {
            path,
            min_free_gb,
            max_used_gb,
        }
    }

    pub fn from_scheme(scheme: DiskSpaceConditionScheme) -> Self {
        Self {
            path: scheme.path,
            min_free_gb: scheme.min_free_gb,
            max_used_gb: scheme.max_used_gb,
        }
    }
}

impl Condition for DiskSpaceCondition {
    fn check(&self) -> bool {
        let _sys = System::new_all();
        let disks = Disks::new_with_refreshed_list();
        for disk in &disks {
            let disk_path = disk.mount_point().to_string_lossy();

            // Check if this disk matches our target path
            if disk_path
                .to_lowercase()
                .starts_with(&self.path.to_lowercase())
            {
                let available_gb = disk.available_space() as f64 / (1024.0 * 1024.0 * 1024.0);

                // Check minimum free space
                if available_gb < self.min_free_gb {
                    return false;
                }

                // Check maximum used space if specified
                if let Some(max_used) = self.max_used_gb {
                    let total_gb = disk.total_space() as f64 / (1024.0 * 1024.0 * 1024.0);
                    let used_gb = total_gb - available_gb;
                    if used_gb > max_used {
                        return false;
                    }
                }

                return true;
            }
        }

        // If we get here, the specified path wasn't found
        false
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "Disk Space"
    }

    fn create(&self) -> Result<ConditionScheme, AutoPilotError> {
        let path = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter path to check disk space (e.g., /, C:\\, /home):")
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let min_free_gb: f64 = Input::<f64>::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter minimum free space required in GB:")
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;
        // .parse()
        // .map_err(|_| AutoPilotError::Condition("Invalid number format".to_string()))?;

        let max_used_gb_input: f64 = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter maximum used space allowed in GB (leave empty for no limit):")
            .default(0.)
            // .allow_empty(true)
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let max_used_gb = if max_used_gb_input == 0.0 {
            None
        } else {
            Some(
                max_used_gb_input, // .parse()
                                   // .map_err(|_| AutoPilotError::Condition("Invalid number format".to_string()))?,
            )
        };

        Ok(ConditionScheme::DiskSpace(DiskSpaceConditionScheme {
            path,
            min_free_gb,
            max_used_gb,
        }))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct DiskSpaceConditionScheme {
    #[serde(default)]
    pub path: String,
    #[serde(default)]
    pub min_free_gb: f64,
    #[serde(default)]
    pub max_used_gb: Option<f64>,
}
