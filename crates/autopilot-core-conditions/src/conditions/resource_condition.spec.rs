use super::*;

#[test]
fn from_scheme_normalises_resource_and_operator() {
    let condition = ResourceCondition::from_scheme(ResourceConditionScheme {
        resource_type: "CPU".to_string(),
        threshold: 10.0,
        operator: None,
    });
    let memory = ResourceCondition::from_scheme(ResourceConditionScheme {
        resource_type: "RAM".to_string(),
        threshold: 10.0,
        operator: Some("LESS".to_string()),
    });

    assert_eq!(condition.resource_type, "cpu");
    assert_eq!(condition.operator, "greater");
    assert_eq!(memory.resource_type, "ram");
    assert_eq!(memory.operator, "less");
}

#[test]
fn evaluates_greater_and_less_operators() {
    let below_100 = ResourceCondition::new("cpu".to_string(), 101.0, "less".to_string());
    let above_100 = ResourceCondition::new("cpu".to_string(), 101.0, "greater".to_string());
    let memory_below_100 = ResourceCondition::new("memory".to_string(), 101.0, "lt".to_string());

    assert!(below_100.check());
    assert!(!above_100.check());
    assert!(memory_below_100.check());
}

#[test]
fn unknown_resource_or_operator_fails() {
    let unknown_resource = ResourceCondition::new("gpu".to_string(), 0.0, "greater".to_string());
    let unknown_operator = ResourceCondition::new("cpu".to_string(), 0.0, "equal".to_string());

    assert!(!unknown_resource.check());
    assert!(!unknown_operator.check());
    assert_eq!(unknown_resource.name(), "Resource");
    assert_eq!(unknown_resource.clone_box().name(), "Resource");
}
