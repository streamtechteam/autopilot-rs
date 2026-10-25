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
fn short_process_names_do_not_match_longer_names_in_reverse() {
    assert!(process_name_matches("bash", "bash"));
    assert!(process_name_matches("chromium", "chrom"));
    assert!(!process_name_matches("ssh", "sshd"));
}

#[cfg(target_os = "linux")]
#[test]
fn truncated_linux_comm_names_match_longer_configured_names() {
    assert_eq!("telegram-deskto".len(), 15);
    assert!(process_name_matches("telegram-deskto", "telegram-desktop"));
}

#[test]
fn reports_name_and_clones() {
    let condition = ProcessCondition::new("bash".to_string(), true);

    assert_eq!(condition.name(), "Process");
    assert_eq!(condition.clone_box().name(), "Process");
}
