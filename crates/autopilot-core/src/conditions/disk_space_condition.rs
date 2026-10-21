use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};
use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use sysinfo::Disks;

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
            .with_prompt("Disk mount point (e.g. /, /home, C:)")
            .validate_with(|input: &String| {
                if input.trim().is_empty() {
                    Err("Enter a disk or mount point")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;
        let path = path.trim().to_string();

        let min_free_gb: f64 = Input::<f64>::with_theme(&ColorfulTheme::default())
            .with_prompt("Required free space (GB; fails below this)")
            .validate_with(|input: &f64| {
                if !input.is_finite() || *input < 0.0 {
                    Err("Enter 0 or more")
                } else {
                    Ok(())
                }
            })
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let limit_used = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Also limit used space?")
            .default(false)
            .interact_opt()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?
            .unwrap_or(false);

        let max_used_gb = if limit_used {
            Some(
                Input::<f64>::with_theme(&ColorfulTheme::default())
                    .with_prompt("Max used space (GB; fails above this)")
                    .validate_with(|input: &f64| {
                        if !input.is_finite() || *input < 0.0 {
                            Err("Enter 0 or more")
                        } else {
                            Ok(())
                        }
                    })
                    .interact_text()
                    .map_err(|err| AutoPilotError::Condition(err.to_string()))?,
            )
        } else {
            None
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
