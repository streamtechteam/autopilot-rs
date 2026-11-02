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
}

impl ExternalDeviceCondition {
    pub fn new(device_identifier: String, check_by_name: bool) -> Self {
        Self {
            device_identifier: device_identifier.to_lowercase(),
            check_by_name,
        }
    }

    pub fn from_scheme(scheme: ExternalDeviceConditionScheme) -> Self {
        Self {
            device_identifier: scheme.device_identifier.to_lowercase(),
            check_by_name: scheme.check_by_name.unwrap_or(false),
        }
    }
}

impl Condition for ExternalDeviceCondition {
    fn check(&self) -> bool {
        let disks = Disks::new_with_refreshed_list();

        for disk in &disks {
            let disk_name = disk.name().to_string_lossy().to_lowercase();
            let mount_point = disk.mount_point().to_string_lossy().to_lowercase();

            let matched = if self.check_by_name {
                name_matches(&disk_name, &self.device_identifier)
            } else {
                mount_matches(&mount_point, &self.device_identifier)
            };

            if matched {
                return true;
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
}

/// A device name matches when it contains the identifier.
///
/// The comparison is one-way so short/empty disk names cannot match every
/// identifier.
fn name_matches(disk_name: &str, identifier: &str) -> bool {
    !disk_name.is_empty() && !identifier.is_empty() && disk_name.contains(identifier)
}

/// Mount points must be equal after ignoring a trailing separator, so the root
/// mount cannot match an arbitrary absolute path.
fn mount_matches(mount_point: &str, identifier: &str) -> bool {
    fn normalize(value: &str) -> &str {
        let trimmed = value.trim_end_matches(['/', '\\']);
        if trimmed.is_empty() { value } else { trimmed }
    }

    !mount_point.is_empty()
        && !identifier.is_empty()
        && normalize(mount_point) == normalize(identifier)
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
    #[schemars(extend("default" = false))]
    pub check_by_name: Option<bool>,
}

#[cfg(test)]
#[path = "external_device_condition.spec.rs"]
mod spec;
