use serde::{Deserialize, Serialize};

use crate::conditions::Condition;

/// A condition that always returns false, used as a fallback for error cases
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FailCondition;

impl Condition for FailCondition {
    fn check(&self) -> bool {
        false
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(FailCondition)
    }
}
