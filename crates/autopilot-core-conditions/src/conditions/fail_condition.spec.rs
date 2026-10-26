use super::*;

#[test]
fn fail_condition_always_fails() {
    assert!(!FailCondition.check());
    assert_eq!(FailCondition.name(), "Fail");
    assert_eq!(FailCondition.clone_box().name(), "Fail");
}

#[test]
fn fail_condition_default_is_constructible() {
    let condition = <FailCondition as Default>::default();
    assert!(!condition.check());
}
