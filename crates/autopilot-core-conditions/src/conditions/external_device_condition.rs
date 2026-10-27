use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use sysinfo::Disks;

use crate::conditions::Condition;

/// Represents an external device (USB/mount) condition
#[derive(Clone)]
pub struct ExternalDeviceCondition {
    /// Name or mount point of the device to check for
    pub device_identifier: String,
    /// Whether to check by name (true) or mount point (false)
    pub check_by_name: bool,
    /// Per-condition polling interval override in milliseconds.
    /// `None` means the job-level `check_interval` applies.
    pub check_interval: Option<String>,
}

impl ExternalDeviceCondition {
    pub fn new(device_identifier: String, check_by_name: bool) -> Self {
        Self {
            device_identifier: device_identifier.to_lowercase(),
            check_by_name,
            check_interval: None,
        }
    }

    pub fn from_scheme(scheme: ExternalDeviceConditionScheme) -> Self {
        Self {
            device_identifier: scheme.device_identifier.to_lowercase(),
            check_by_name: scheme.check_by_name.unwrap_or(false),
            check_interval: scheme.check_interval,
        }
    }
}

impl Condition for ExternalDeviceCondition {
    fn check(&self) -> bool {
        let disks = Disks::new_with_refreshed_list();

        for disk in &disks {
            let disk_name = disk.name().to_string_lossy().to_lowercase();
            let mount_point = disk.mount_point().to_string_lossy().to_lowercase();

            if self.check_by_name {
                // Check if the disk name contains our identifier
                if disk_name.contains(&self.device_identifier)
                    || self.device_identifier.contains(&disk_name)
                {
                    return true;
                }
            } else {
                // Check if the mount point contains our identifier
                if mount_point.contains(&self.device_identifier)
                    || self.device_identifier.contains(&mount_point)
                {
                    return true;
                }
            }
        }

        false
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "External Device"
    }
    fn check_interval(&self) -> Option<&str> {
        self.check_interval.as_deref()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "External device condition")]
pub struct ExternalDeviceConditionScheme {
    #[serde(default)]
    #[schemars(
        title = "Device",
        description = "Device name or mount point",
        extend(
            "x-autopilot" = {
                "required": true,
                "widget": "dynamic-select",
                "provider": "disks",
                "allow_custom": true
            }
        )
    )]
    pub device_identifier: String,
    #[serde(default)]
    #[schemars(title = "Check by device name")]
    pub check_by_name: Option<bool>,
    #[serde(default)]
    #[schemars(
        title = "Check interval",
        description = "Milliseconds between checks. Overrides the job-level check_interval."
    )]
    pub check_interval: Option<String>,
}

#[cfg(test)]
#[path = "external_device_condition.spec.rs"]
mod spec;
