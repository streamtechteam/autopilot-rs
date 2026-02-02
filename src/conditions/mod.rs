use serde::{Deserialize, Serialize};
use log::error;

pub mod output_condition;
pub mod variable_condition;
pub mod time_condition;
pub mod bluetooth_condition;
pub mod custom_condition;
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
    /// Time condition: checks if the current time matches a target datetime
    Time(time_condition::TimeConditionScheme),
    /// Bluetooth condition: checks if a Bluetooth device is connected
    Bluetooth(bluetooth_condition::BluetoothConditionScheme),
    /// Custom condition: executes a custom shell command and checks the result
    Custom(custom_condition::CustomConditionScheme),
    /// WiFi condition: checks if connected to a specific WiFi network
    Wifi(wifi_condition::WifiConditionScheme),
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
            ConditionScheme::Time(scheme) => match time_condition::TimeCondition::from_scheme(scheme.clone()) {
                Ok(condition) => Box::new(condition),
                Err(e) => {
                    error!("Error parsing time condition: {}", e);
                    // Return a condition that always fails
                    Box::new(FailCondition)
                }
            },
            ConditionScheme::Bluetooth(scheme) => Box::new(
                bluetooth_condition::BluetoothCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::Custom(scheme) => Box::new(
                custom_condition::CustomCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::Wifi(scheme) => Box::new(
                wifi_condition::WifiCondition::from_scheme(scheme.clone()),
            ),
        }
    }
}

/// A condition that always returns false, used as a fallback for error cases
#[derive(Clone)]
struct FailCondition;

impl Condition for FailCondition {
    fn check(&self) -> bool {
        false
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(FailCondition)
    }
}
