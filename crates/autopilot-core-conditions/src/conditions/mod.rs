use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use strum::{EnumIter, IntoEnumIterator};

use autopilot_core_form::{ConditionConfig, FormError, FormSchema, FormValues};

pub mod bluetooth_condition;
pub mod command_condition;
pub mod custom_condition;
pub mod desktop_env_condition;
pub mod disk_space_condition;
pub mod external_device_condition;
pub mod fail_condition;
pub mod file_condition;
pub mod internet_condition;
pub mod interval_override;
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

    /// Per-condition polling interval override in milliseconds, as configured.
    /// `None` means the job-level `check_interval` applies.
    fn check_interval(&self) -> Option<&str> {
        None
    }
}

/// Implement Clone for Box<dyn Condition>
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

impl ConditionScheme {
    pub fn form_types() -> Vec<ConditionFormType> {
        vec![
            ConditionFormType {
                key: "command",
                label: "Command",
            },
            ConditionFormType {
                key: "variable",
                label: "Variable",
            },
            ConditionFormType {
                key: "desktopenv",
                label: "Desktop Environment",
            },
            ConditionFormType {
                key: "bluetooth",
                label: "Bluetooth",
            },
            ConditionFormType {
                key: "wifi",
                label: "WiFi",
            },
            ConditionFormType {
                key: "power",
                label: "Power",
            },
            ConditionFormType {
                key: "resource",
                label: "Resource",
            },
            ConditionFormType {
                key: "internet",
                label: "Internet",
            },
            ConditionFormType {
                key: "process",
                label: "Process",
            },
            ConditionFormType {
                key: "diskspace",
                label: "Disk Space",
            },
            ConditionFormType {
                key: "file",
                label: "File",
            },
            ConditionFormType {
                key: "externaldevice",
                label: "External Device",
            },
            ConditionFormType {
                key: "screen",
                label: "Screen",
            },
            ConditionFormType {
                key: "logical",
                label: "Logical",
            },
        ]
    }

    pub fn form_type_key(&self) -> &'static str {
        match self {
            ConditionScheme::Command(_) => "command",
            ConditionScheme::Variable(_) => "variable",
            ConditionScheme::DesktopEnv(_) => "desktopenv",
            ConditionScheme::Bluetooth(_) => "bluetooth",
            ConditionScheme::Wifi(_) => "wifi",
            ConditionScheme::Power(_) => "power",
            ConditionScheme::Resource(_) => "resource",
            ConditionScheme::Internet(_) => "internet",
            ConditionScheme::Process(_) => "process",
            ConditionScheme::DiskSpace(_) => "diskspace",
            ConditionScheme::File(_) => "file",
            ConditionScheme::ExternalDevice(_) => "externaldevice",
            ConditionScheme::Fail(_) => "fail",
            ConditionScheme::Screen(_) => "screen",
            ConditionScheme::Logical(_) => "logical",
        }
    }

    pub fn to_form_values(&self) -> Result<FormValues, FormError> {
        match self {
            ConditionScheme::Command(scheme) => scheme.to_form_values(),
            ConditionScheme::Variable(scheme) => scheme.to_form_values(),
            ConditionScheme::DesktopEnv(scheme) => scheme.to_form_values(),
            ConditionScheme::Bluetooth(scheme) => scheme.to_form_values(),
            ConditionScheme::Wifi(scheme) => scheme.to_form_values(),
            ConditionScheme::Power(scheme) => scheme.to_form_values(),
            ConditionScheme::Resource(scheme) => scheme.to_form_values(),
            ConditionScheme::Internet(scheme) => scheme.to_form_values(),
            ConditionScheme::Process(scheme) => scheme.to_form_values(),
            ConditionScheme::DiskSpace(scheme) => scheme.to_form_values(),
            ConditionScheme::File(scheme) => scheme.to_form_values(),
            ConditionScheme::ExternalDevice(scheme) => scheme.to_form_values(),
            ConditionScheme::Fail(_) => Err(FormError::Unsupported(
                "fail conditions cannot be created from a form".to_string(),
            )),
            ConditionScheme::Screen(scheme) => scheme.to_form_values(),
            ConditionScheme::Logical(scheme) => scheme.to_form_values(),
        }
    }

    pub fn form_schema(key: &str) -> Result<FormSchema, FormError> {
        match key {
            "command" => command_condition::CommandConditionScheme::try_form_schema(),
            "variable" => variable_condition::VariableConditionScheme::try_form_schema(),
            "desktopenv" => desktop_env_condition::DesktopEnvConditionScheme::try_form_schema(),
            "bluetooth" => bluetooth_condition::BluetoothConditionScheme::try_form_schema(),
            "wifi" => wifi_condition::WifiConditionScheme::try_form_schema(),
            "power" => power_condition::PowerConditionScheme::try_form_schema(),
            "resource" => resource_condition::ResourceConditionScheme::try_form_schema(),
            "internet" => internet_condition::InternetConditionScheme::try_form_schema(),
            "process" => process_condition::ProcessConditionScheme::try_form_schema(),
            "diskspace" => disk_space_condition::DiskSpaceConditionScheme::try_form_schema(),
            "file" => file_condition::FileConditionScheme::try_form_schema(),
            "externaldevice" => {
                external_device_condition::ExternalDeviceConditionScheme::try_form_schema()
            }
            "screen" => screen_condition::ScreenConditionScheme::try_form_schema(),
            "logical" => logical_condition::LogicalConditionScheme::try_form_schema(),
            other => Err(FormError::Unsupported(format!(
                "unknown condition form type '{other}'"
            ))),
        }
    }

    pub fn from_form_values(key: &str, values: FormValues) -> Result<Self, FormError> {
        match key {
            "command" => Ok(Self::Command(
                command_condition::CommandConditionScheme::from_form_values(values)?,
            )),
            "variable" => Ok(Self::Variable(
                variable_condition::VariableConditionScheme::from_form_values(values)?,
            )),
            "desktopenv" => Ok(Self::DesktopEnv(
                desktop_env_condition::DesktopEnvConditionScheme::from_form_values(values)?,
            )),
            "bluetooth" => Ok(Self::Bluetooth(
                bluetooth_condition::BluetoothConditionScheme::from_form_values(values)?,
            )),
            "wifi" => Ok(Self::Wifi(
                wifi_condition::WifiConditionScheme::from_form_values(values)?,
            )),
            "power" => Ok(Self::Power(
                power_condition::PowerConditionScheme::from_form_values(values)?,
            )),
            "resource" => Ok(Self::Resource(
                resource_condition::ResourceConditionScheme::from_form_values(values)?,
            )),
            "internet" => Ok(Self::Internet(
                internet_condition::InternetConditionScheme::from_form_values(values)?,
            )),
            "process" => Ok(Self::Process(
                process_condition::ProcessConditionScheme::from_form_values(values)?,
            )),
            "diskspace" => Ok(Self::DiskSpace(
                disk_space_condition::DiskSpaceConditionScheme::from_form_values(values)?,
            )),
            "file" => Ok(Self::File(
                file_condition::FileConditionScheme::from_form_values(values)?,
            )),
            "externaldevice" => Ok(Self::ExternalDevice(
                external_device_condition::ExternalDeviceConditionScheme::from_form_values(values)?,
            )),
            "screen" => Ok(Self::Screen(
                screen_condition::ScreenConditionScheme::from_form_values(values)?,
            )),
            "logical" => Ok(Self::Logical(
                logical_condition::LogicalConditionScheme::from_form_values(values)?,
            )),
            other => Err(FormError::Unsupported(format!(
                "unknown condition form type '{other}'"
            ))),
        }
    }

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
            ConditionScheme::Logical(scheme) => Box::new(
                logical_condition::LogicalCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::DesktopEnv(scheme) => Box::new(
                desktop_env_condition::DesktopEnvCondition::from_scheme(scheme.clone()),
            ),
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
}

macro_rules! impl_condition_config {
    ($($type:ty),+ $(,)?) => {
        $(impl ConditionConfig for $type {})+
    };
}

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

//  TODO
// Absolutly terrifing module, was to do everything manually and its fucking bad
