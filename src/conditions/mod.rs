use serde::{Deserialize, Serialize};
use tokio_cron_scheduler::JobScheduler;

pub mod output_condition;

pub mod variable_condition;

pub mod time_condition;

pub mod de_condition;

pub mod bluetooth_condition;

pub mod custom_condition;

pub mod wifi_condition;

pub trait Condition: Send + Sync {
    fn check(&self) -> bool;
    // fn new(&self) -> Self;
    
    // Method to support cloning through trait objects
    fn clone_box(&self) -> Box<dyn Condition>;
}

// Implement Clone for Box<dyn Condition>
impl Clone for Box<dyn Condition> {
    fn clone(&self) -> Box<dyn Condition> {
        self.clone_box()
    }
}

// pub trait Time_Condition: Send + Sync {
//     fn check(&self , scheduler: &JobScheduler) -> bool;
//     // fn new(&self) -> Self;
// }

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "type", content = "condition", rename_all = "lowercase")]
pub enum ConditionScheme {
    Output(output_condition::OutputConditionScheme),
    Variable(variable_condition::VariableConditionScheme),
    // Time(time_condition::TimeConditionScheme),
    // Device(device_condition::DeviceCondition),
    // Bluetooth(bluetooth_condition::BluetoothCondition),
    // Custom(custom_condition::CustomCondition),
    // Wifi(wifi_condition::WifiCondition),
}

impl ConditionScheme {
    pub fn to_condition(&self, scheduler: &JobScheduler) -> Box<dyn Condition> {
        match self {
            ConditionScheme::Output(scheme) => Box::new(
                output_condition::OutputCondition::from_scheme(scheme.clone()),
            ),
            ConditionScheme::Variable(scheme) => Box::new(
                variable_condition::VariableCondition::from_scheme(scheme.clone()),
            ),
            // ConditionScheme::Time(scheme) => Box::new(
            //     time_condition::TimeCondition::from_scheme(scheme.clone(), scheduler)
            //         .expect("Error while processing condition schemes"),
            // ),
        }
    }
}

// #[derive(Debug, Serialize, Deserialize)]
// pub struct ConditionScheme {
//     pub name: String,
//     pub condition: ConditionType,
// }
