use super::*;

#[test]
fn lowercases_process_name_when_constructed() {
    let condition = ProcessCondition::new("Bash".to_string(), true);

    assert_eq!(condition.process_name, "bash");
    assert!(condition.should_be_running);
}

#[test]
fn from_scheme_defaults_to_should_be_running() {
    let condition = ProcessCondition::from_scheme(ProcessConditionScheme {
        process_name: "BASH".to_string(),
        should_be_running: None,
    });

    assert_eq!(condition.process_name, "bash");
    assert!(condition.should_be_running);
}

#[test]
fn detects_the_current_process_by_comm_name() {
    let current = std::fs::read_to_string(format!("/proc/{}/comm", std::process::id())).unwrap();
    let current = current.trim();
    let condition = ProcessCondition::new(current.to_uppercase(), true);

    assert!(condition.check());
    assert!(!ProcessCondition::new(current.to_string(), false).check());
}

#[test]
fn missing_process_is_not_running() {
    let condition = ProcessCondition::new(
        "autopilot-spec-process-that-does-not-exist".to_string(),
        true,
    );

    assert!(!condition.check());
    assert!(
        ProcessCondition::new(
            "autopilot-spec-process-that-does-not-exist".to_string(),
            false
        )
        .check()
    );
}

#[test]
fn reports_name_and_clones() {
    let condition = ProcessCondition::new("bash".to_string(), true);

    assert_eq!(condition.name(), "Process");
    assert_eq!(condition.clone_box().name(), "Process");
}
