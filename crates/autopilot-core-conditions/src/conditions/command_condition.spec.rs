use super::*;

#[test]
fn passes_on_zero_exit_code() {
    assert!(CommandCondition::new("echo ok".to_string()).check());
}

#[test]
fn fails_on_non_zero_exit_code() {
    assert!(!CommandCondition::new("exit 3".to_string()).check());
}

#[test]
fn compares_command_output() {
    let matching = CommandCondition::with_output("echo hello".to_string(), "hello".to_string());
    let mismatched = CommandCondition::with_output("echo hello".to_string(), "world".to_string());

    assert!(matching.check());
    assert!(!mismatched.check());
}

#[test]
fn from_scheme_defaults_to_exit_code_check() {
    let condition = CommandCondition::from_scheme(CommandConditionScheme {
        command: "echo ok".to_string(),
        check_exit_code: None,
        target_output: Some("ignored".to_string()),
        check_interval: None,
    });

    assert!(condition.check_exit_code);
    assert_eq!(condition.target_output, Some("ignored".to_string()));
}

#[test]
fn reports_name_and_clones() {
    let condition = CommandCondition::new("echo ok".to_string());

    assert_eq!(condition.name(), "Command");
    assert_eq!(condition.clone_box().name(), "Command");
}
