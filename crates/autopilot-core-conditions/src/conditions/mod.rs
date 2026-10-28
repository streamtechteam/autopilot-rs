use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::EnumIter;

use autopilot_core_form::{
    ConditionConfig, FormError, FormSchema, FormValues, impl_condition_config,
};

pub mod bluetooth_condition;
pub mod command_condition;
pub mod custom_condition;
pub mod desktop_env_condition;
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

    fn name(&self) -> &str;
}

/// Implement `Clone` for `Box<dyn Condition>`
impl Clone for Box<dyn Condition> {
    fn clone(&self) -> Box<dyn Condition> {
        self.clone_box()
    }
}

/// Unified enum for all condition types, supporting deserialization from JSON/JSONC
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, EnumIter)]
#[serde(tag = "type", content = "condition", rename_all = "lowercase")]
pub enum ConditionScheme {
    /// Command condition: checks if command output matches a target value
    Command(command_condition::CommandConditionScheme),
    /// Variable condition: checks if an environment variable matches a target value
    Variable(variable_condition::VariableConditionScheme),
    DesktopEnv(desktop_env_condition::DesktopEnvConditionScheme),
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
    // And(and_condition::AndConditionScheme),
    // Or(or_condition::OrConditionScheme),
    // Nor(nor_condition::NorConditionScheme),
    Screen(screen_condition::ScreenConditionScheme),
    Logical(logical_condition::LogicalConditionScheme),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct ConditionFormType {
    pub key: &'static str,
    pub label: &'static str,
}

mod registry;

// Each entry declares a condition once: enum variant, form key, form label,
// config type, and runtime type. Conditions that cannot be built from a form go
// in `no_form` and only need a key.
registry::condition_registry! {
    form {
        Command => {
            key: "command",
            label: "Command",
            scheme: command_condition::CommandConditionScheme,
            condition: command_condition::CommandCondition,
        };
        Variable => {
            key: "variable",
            label: "Variable",
            scheme: variable_condition::VariableConditionScheme,
            condition: variable_condition::VariableCondition,
        };
        DesktopEnv => {
            key: "desktopenv",
            label: "Desktop Environment",
            scheme: desktop_env_condition::DesktopEnvConditionScheme,
            condition: desktop_env_condition::DesktopEnvCondition,
        };
        Bluetooth => {
            key: "bluetooth",
            label: "Bluetooth",
            scheme: bluetooth_condition::BluetoothConditionScheme,
            condition: bluetooth_condition::BluetoothCondition,
        };
        Wifi => {
            key: "wifi",
            label: "WiFi",
            scheme: wifi_condition::WifiConditionScheme,
            condition: wifi_condition::WifiCondition,
        };
        Power => {
            key: "power",
            label: "Power",
            scheme: power_condition::PowerConditionScheme,
            condition: power_condition::PowerCondition,
        };
        Resource => {
            key: "resource",
            label: "Resource",
            scheme: resource_condition::ResourceConditionScheme,
            condition: resource_condition::ResourceCondition,
        };
        Internet => {
            key: "internet",
            label: "Internet",
            scheme: internet_condition::InternetConditionScheme,
            condition: internet_condition::InternetCondition,
        };
        Process => {
            key: "process",
            label: "Process",
            scheme: process_condition::ProcessConditionScheme,
            condition: process_condition::ProcessCondition,
        };
        DiskSpace => {
            key: "diskspace",
            label: "Disk Space",
            scheme: disk_space_condition::DiskSpaceConditionScheme,
            condition: disk_space_condition::DiskSpaceCondition,
        };
        File => {
            key: "file",
            label: "File",
            scheme: file_condition::FileConditionScheme,
            condition: file_condition::FileCondition,
        };
        ExternalDevice => {
            key: "externaldevice",
            label: "External Device",
            scheme: external_device_condition::ExternalDeviceConditionScheme,
            condition: external_device_condition::ExternalDeviceCondition,
        };
        Screen => {
            key: "screen",
            label: "Screen",
            scheme: screen_condition::ScreenConditionScheme,
            condition: screen_condition::ScreenCondition,
        };
        Logical => {
            key: "logical",
            label: "Logical",
            scheme: logical_condition::LogicalConditionScheme,
            condition: logical_condition::LogicalCondition,
        };
    }
    no_form {
        Fail => {
            key: "fail",
        };
    }
}

// Most schemes use the default `ConditionConfig` behavior.
// `ScreenConditionScheme` implements the trait in its own module because it
// performs custom validation.
impl_condition_config!(
    command_condition::CommandConditionScheme,
    variable_condition::VariableConditionScheme,
    desktop_env_condition::DesktopEnvConditionScheme,
    bluetooth_condition::BluetoothConditionScheme,
    wifi_condition::WifiConditionScheme,
    power_condition::PowerConditionScheme,
    resource_condition::ResourceConditionScheme,
    internet_condition::InternetConditionScheme,
    process_condition::ProcessConditionScheme,
    disk_space_condition::DiskSpaceConditionScheme,
    file_condition::FileConditionScheme,
    external_device_condition::ExternalDeviceConditionScheme,
    logical_condition::LogicalConditionScheme,
);

#[cfg(test)]
#[path = "conditions.spec.rs"]
mod spec;
