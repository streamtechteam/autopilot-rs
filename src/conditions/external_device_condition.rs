use crate::{
    conditions::{Condition, ConditionScheme},
    error::AutoPilotError,
};
use dialoguer::{Confirm, Input, theme::ColorfulTheme};
use serde::{Deserialize, Serialize};
use sysinfo::{Disks, System};

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
        let sys = System::new_all();
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

    fn create(&self) -> Result<ConditionScheme, AutoPilotError> {
        let device_identifier = Input::with_theme(&ColorfulTheme::default())
            .with_prompt("Enter device name or mount point to check for:")
            .interact_text()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?;

        let check_by_name = Confirm::with_theme(&ColorfulTheme::default())
            .with_prompt("Check by device name? (Otherwise check by mount point)")
            .interact_opt()
            .map_err(|err| AutoPilotError::Condition(err.to_string()))?
            .unwrap_or(true);

        Ok(ConditionScheme::ExternalDevice(
            ExternalDeviceConditionScheme {
                device_identifier,
                check_by_name: Some(check_by_name),
            },
        ))
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, Default)]
#[serde(deny_unknown_fields)]
pub struct ExternalDeviceConditionScheme {
    #[serde(default)]
    pub device_identifier: String,
    #[serde(default)]
    pub check_by_name: Option<bool>,
}
