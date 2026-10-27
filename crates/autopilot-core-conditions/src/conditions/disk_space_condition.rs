use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sysinfo::Disks;

use crate::conditions::Condition;

/// Represents a disk space condition
#[derive(Clone)]
pub struct DiskSpaceCondition {
    /// Path to the disk/mount point to check (e.g., "/", "C:\\", "/home")
    pub path: String,
    /// Minimum free space required in GB
    pub min_free_gb: f64,
    /// Maximum used space allowed in GB
    pub max_used_gb: Option<f64>,
    /// Per-condition polling interval override in milliseconds.
    /// `None` means the job-level `check_interval` applies.
    pub check_interval: Option<String>,
}

impl DiskSpaceCondition {
    pub fn new(path: String, min_free_gb: f64, max_used_gb: Option<f64>) -> Self {
        Self {
            path,
            min_free_gb,
            max_used_gb,
            check_interval: None,
        }
    }

    pub fn from_scheme(scheme: DiskSpaceConditionScheme) -> Self {
        Self {
            path: scheme.path,
            min_free_gb: scheme.min_free_gb,
            max_used_gb: scheme.max_used_gb,
            check_interval: scheme.check_interval,
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
    fn check_interval(&self) -> Option<&str> {
        self.check_interval.as_deref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Disk space condition")]
pub struct DiskSpaceConditionScheme {
    #[serde(default)]
    #[schemars(
        title = "Mount point",
        extend(
            "x-autopilot" = {
                "required": true,
                "widget": "dynamic-select",
                "provider": "disks",
                "allow_custom": true
            }
        )
    )]
    pub path: String,
    #[serde(default)]
    #[schemars(
        title = "Minimum free space",
        range(min = 0.0),
        extend("x-autopilot" = {"required": true, "unit": "GB"})
    )]
    pub min_free_gb: f64,
    #[serde(default)]
    #[schemars(
        title = "Maximum used space",
        range(min = 0.0),
        extend("x-autopilot" = {"unit": "GB"})
    )]
    pub max_used_gb: Option<f64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[schemars(
        title = "Check interval",
        description = "Milliseconds between checks. Overrides the job-level check_interval."
    )]
    pub check_interval: Option<String>,
}

#[cfg(test)]
#[path = "disk_space_condition.spec.rs"]
mod spec;
