use log::error;
use serde::{Deserialize, Serialize};

pub mod bluetooth_condition;
pub mod custom_condition;
pub mod disk_space_condition;
pub mod external_device_condition;
pub mod fail_condition;
pub mod file_condition;
pub mod internet_condition;
pub mod output_condition;
pub mod power_condition;
pub mod process_condition;
pub mod resource_condition;
pub mod variable_condition;
pub mod wifi_condition;

/// Base trait for all condition types
pub trait Condition: Send + Sync {
    /// Check if the condition is satisfied
    fn check(&self) -> bool;

    /// Method to support cloning through trait objects
    fn clone_box(&self) -> Box<dyn Condition>;
}

/// Implement Clone for Box<dyn Condition>
impl Clone for Box<dyn Condition> {
    fn clone(&self) -> Box<dyn Condition> {
        self.clone_box()
    }
}

/// Unified enum for all condition types, supporting deserialization from JSON/JSONC
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "condition", rename_all = "lowercase")]
pub enum ConditionScheme {
    /// Output condition: checks if command output matches a target value
    Output(output_condition::OutputConditionScheme),
    /// Variable condition: checks if an environment variable matches a target value
    Variable(variable_condition::VariableConditionScheme),
    /// Bluetooth condition: checks if a Bluetooth device is connected
    Bluetooth(bluetooth_condition::BluetoothConditionScheme),
    /// Custom condition: executes a custom shell command and checks the result
    Custom(custom_condition::CustomConditionScheme),
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
}

impl ConditionScheme {
    /// Convert a ConditionScheme into a boxed Condition trait object
    pub fn to_condition(&self) -> Box<dyn Condition> {
        match self {
            ConditionScheme::Output(scheme) => Box::new(
                output_condition::OutputCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::Variable(scheme) => Box::new(
                variable_condition::VariableCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::Bluetooth(scheme) => Box::new(
                bluetooth_condition::BluetoothCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::Custom(scheme) => Box::new(
                custom_condition::CustomCondition::from_scheme(scheme.clone()),
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
        }
    }
}
