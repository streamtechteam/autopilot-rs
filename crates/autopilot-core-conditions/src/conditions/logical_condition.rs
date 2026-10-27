use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::conditions::{Condition, ConditionScheme};

#[derive(Clone)]
pub struct LogicalCondition {
    operator: LogicalOperator,
    conditions: Vec<Box<dyn Condition>>,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Default, Deserialize, Serialize, JsonSchema)]
pub enum LogicalOperator {
    #[default]
    And,
    Or,
    Nor,
}

impl LogicalOperator {
    #[cfg(test)]
    fn name(&self) -> &'static str {
        match self {
            LogicalOperator::And => "AND",
            LogicalOperator::Or => "OR",
            LogicalOperator::Nor => "NOR",
        }
    }

    fn evaluate(&self, results: &[bool]) -> bool {
        match self {
            LogicalOperator::And => results.iter().all(|&x| x),
            LogicalOperator::Or => results.iter().any(|&x| x),
            LogicalOperator::Nor => !results.iter().any(|&x| x), // NOT (A OR B OR C...)
        }
    }
}

impl LogicalCondition {
    pub fn new(operator: LogicalOperator, conditions: Vec<Box<dyn Condition>>) -> Self {
        LogicalCondition {
            operator,
            conditions,
        }
    }

    pub fn from_scheme(scheme: LogicalConditionScheme) -> Self {
        LogicalCondition {
            operator: scheme.operator,
            conditions: scheme
                .conditions
                .into_iter()
                .map(|c| c.to_condition())
                .collect::<Vec<_>>(),
        }
    }
}

impl Condition for LogicalCondition {
    fn check(&self) -> bool {
        let results: Vec<bool> = self
            .conditions
            .iter()
            .map(|condition| condition.check())
            .collect();

        self.operator.evaluate(&results)
    }

    fn clone_box(&self) -> Box<dyn Condition> {
        Box::new(self.clone())
    }

    fn name(&self) -> &str {
        "Logical"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize, Default, JsonSchema)]
#[serde(deny_unknown_fields)]
#[schemars(title = "Logical condition")]
pub struct LogicalConditionScheme {
    #[serde(rename = "operator")]
    #[schemars(
        title = "Operator",
        extend("x-autopilot" = {"required": true})
    )]
    operator: LogicalOperator,

    #[serde(default)]
    #[schemars(
        title = "Conditions",
        extend(
            "x-autopilot" = {
                "required": true,
                "widget": "conditions",
                "min_items": 1
            }
        )
    )]
    conditions: Vec<ConditionScheme>,
}

#[cfg(test)]
#[path = "logical_condition.spec.rs"]
mod spec;
