use super::*;

#[test]
fn compares_matching_and_missing_variables() {
    let key = "AUTOPILOT_SPEC_VARIABLE_MATCH";
    // SAFETY: the variable name is unique to this test.
    unsafe { std::env::set_var(key, "expected") };

    assert!(sync_condition(key, "expected"));
    assert!(!sync_condition(key, "other"));
    assert!(!sync_condition(
        "AUTOPILOT_SPEC_VARIABLE_DOES_NOT_EXIST",
        "value"
    ));

    // SAFETY: the variable was created by this test.
    unsafe { std::env::remove_var(key) };
}

#[test]
fn async_condition_matches_sync_condition() {
    let key = "AUTOPILOT_SPEC_VARIABLE_ASYNC";
    // SAFETY: the variable name is unique to this test.
    unsafe { std::env::set_var(key, "value") };

    assert!(futures::executor::block_on(async_condition(key, "value")));
    assert!(!futures::executor::block_on(async_condition(key, "nope")));

    // SAFETY: the variable was created by this test.
    unsafe { std::env::remove_var(key) };
}

#[test]
fn condition_uses_scheme_values() {
    let condition = VariableCondition::from_scheme(VariableConditionScheme {
        variable: "SOME_VAR".to_string(),
        target: "some-value".to_string(),
    });

    assert_eq!(condition.variable, "SOME_VAR");
    assert_eq!(condition.target, "some-value");
    assert_eq!(condition.name(), "Variable");
    assert_eq!(condition.clone_box().name(), "Variable");
}
