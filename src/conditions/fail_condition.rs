use serde::{Deserialize, Serialize};

use crate::{conditions::Condition, error::AutoPilotError};

/// A condition that always returns false, used as a fallback for error cases
#[derive(Clone, Debug, Deserialize, Serialize, Default)]
pub struct FailCondition;

impl Condition for FailCondition {
    fn check(&self) -> bool {
        false
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(FailCondition)
    }

    fn name(&self) -> &str {
        "Fail"
    }

    fn create(&self) -> Result<crate::conditions::ConditionScheme, AutoPilotError> {
        // This is a fallback condition, so we'll return an error to indicate that
        // this condition should not be created interactively
        Err(AutoPilotError::Condition("Fail condition is a fallback and should not be created interactively".to_string()))
    }
}
