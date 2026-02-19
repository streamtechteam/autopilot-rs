
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use crate::error::AutoPilotError;

pub mod bluetooth_condition;
pub mod command_condition;
pub mod custom_condition;
pub mod de_condition;
pub mod disk_space_condition;
pub mod external_device_condition;
pub mod fail_condition;
pub mod file_condition;
pub mod internet_condition;
pub mod logical_condition;
pub mod power_condition;
pub mod process_condition;
pub mod resource_condition;
pub mod screen_condition;
pub mod variable_condition;
pub mod wifi_condition;

/// Base trait for all condition types
pub trait Condition: Send + Sync {
    /// Check if the condition is satisfied
    fn check(&self) -> bool;

    /// Method to support cloning through trait objects
    fn clone_box(&self) -> Box<dyn Condition>;

    fn create(&self) -> Result<ConditionScheme, AutoPilotError>;

    fn name(&self) -> &str;
}

/// Implement Clone for Box<dyn Condition>
impl Clone for Box<dyn Condition> {
    fn clone(&self) -> Box<dyn Condition> {
        self.clone_box()
    }
}

/// Unified enum for all condition types, supporting deserialization from JSON/JSONC
#[derive(Clone, Debug, Serialize, Deserialize, EnumIter)]
#[serde(tag = "type", content = "condition", rename_all = "lowercase")]
pub enum ConditionScheme {
    /// Command condition: checks if command output matches a target value
    Command(command_condition::CommandConditionScheme),
    /// Variable condition: checks if an environment variable matches a target value
    Variable(variable_condition::VariableConditionScheme),
    /// Bluetooth condition: checks if a Bluetooth device is connected
    Bluetooth(bluetooth_condition::BluetoothConditionScheme),
    /// WiFi condition: checks if connected to a specific WiFi network
    Wifi(wifi_condition::WifiConditionScheme),
    /// Power condition: checks charging status or battery level
    Power(power_condition::PowerConditionScheme),
    /// Resource condition: checks CPU or RAM usage
    Resource(resource_condition::ResourceConditionScheme),
    /// Internet condition: checks internet reachability
    Internet(internet_condition::InternetConditionScheme),
    /// Process condition: checks if a process is running
    Process(process_condition::ProcessConditionScheme),
    /// Disk space condition: checks available disk space
    DiskSpace(disk_space_condition::DiskSpaceConditionScheme),
    /// File condition: checks file existence or properties
    File(file_condition::FileConditionScheme),
    /// External device condition: checks for connected USB/external drives
    ExternalDevice(external_device_condition::ExternalDeviceConditionScheme),
    Fail(fail_condition::FailCondition),
    Logical(logical_condition::LogicalConditionScheme),
    Screen(screen_condition::ScreenConditionScheme),
}

impl ConditionScheme {
    /// Convert a ConditionScheme into a boxed Condition trait object
    pub fn to_condition(&self) -> Box<dyn Condition> {
        match self {
            ConditionScheme::Command(scheme) => Box::new(
                command_condition::CommandCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::Variable(scheme) => Box::new(
                variable_condition::VariableCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::Bluetooth(scheme) => Box::new(
                bluetooth_condition::BluetoothCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::Wifi(scheme) => {
                Box::new(wifi_condition::WifiCondition::from_scheme(scheme.clone()))
            }
            ConditionScheme::Power(scheme) => {
                Box::new(power_condition::PowerCondition::from_scheme(scheme.clone()))
            }
            ConditionScheme::Resource(scheme) => Box::new(
                resource_condition::ResourceCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::Internet(scheme) => Box::new(
                internet_condition::InternetCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::Process(scheme) => Box::new(
                process_condition::ProcessCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::DiskSpace(scheme) => Box::new(
                disk_space_condition::DiskSpaceCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::File(scheme) => {
                Box::new(file_condition::FileCondition::from_scheme(scheme.clone()))
            }
            ConditionScheme::ExternalDevice(scheme) => Box::new(
                external_device_condition::ExternalDeviceCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::Fail(scheme) => Box::new(scheme.clone()),
            ConditionScheme::Logical(scheme) => {
                Box::new(logical_condition::LogicalCondition::from_scheme(scheme.clone()))
            }
            ConditionScheme::Screen(scheme) => Box::new(
                screen_condition::ScreenCondition::from_scheme(scheme.clone()),
            ),
        }
    }

    pub fn varient_names() -> Vec<String> {
        ConditionScheme::iter()
            .map(|variant| {
                // Convert to lowercase to match your serde rename
                variant.to_condition().name().to_string()
            })
            .collect()
    }

    // pub fn get_props(&self)
}
