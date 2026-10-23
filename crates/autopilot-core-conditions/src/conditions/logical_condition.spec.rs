use super::*;
use crate::conditions::{
    command_condition::{CommandCondition, CommandConditionScheme},
    fail_condition::FailCondition,
};

fn fail() -> Box<dyn Condition> {
    Box::new(FailCondition)
}

fn passes() -> Box<dyn Condition> {
    Box::new(CommandCondition::new("exit 0".to_string()))
}

#[test]
fn and_requires_all_conditions() {
    assert!(LogicalCondition::new(LogicalOperator::And, vec![passes(), passes()]).check());
    assert!(!LogicalCondition::new(LogicalOperator::And, vec![passes(), fail()]).check());
    assert!(LogicalCondition::new(LogicalOperator::And, vec![]).check());
}

#[test]
fn or_requires_one_condition() {
    assert!(LogicalCondition::new(LogicalOperator::Or, vec![passes(), fail()]).check());
    assert!(!LogicalCondition::new(LogicalOperator::Or, vec![fail(), fail()]).check());
    assert!(!LogicalCondition::new(LogicalOperator::Or, vec![]).check());
}

#[test]
fn nor_requires_no_condition() {
    assert!(LogicalCondition::new(LogicalOperator::Nor, vec![fail(), fail()]).check());
    assert!(!LogicalCondition::new(LogicalOperator::Nor, vec![passes(), fail()]).check());
    assert!(LogicalCondition::new(LogicalOperator::Nor, vec![]).check());
}

#[test]
fn evaluates_scheme_conditions() {
    let condition = LogicalCondition::from_scheme(LogicalConditionScheme {
        operator: LogicalOperator::Or,
        conditions: vec![
            ConditionScheme::Fail(FailCondition),
            ConditionScheme::Command(CommandConditionScheme {
                command: "exit 0".to_string(),
                check_exit_code: None,
                target_output: None,
            }),
        ],
    });

    assert!(condition.check());
    assert_eq!(condition.operator.name(), "OR");
    assert_eq!(condition.name(), "Logical");
    assert_eq!(condition.clone_box().name(), "Logical");
}

#[test]
fn operator_default_is_and() {
    assert_eq!(LogicalOperator::default(), LogicalOperator::And);
}
