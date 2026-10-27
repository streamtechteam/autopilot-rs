#[cfg(unix)]
use autopilot_core::conditions::disk_space_condition::DiskSpaceCondition;
#[cfg(target_os = "linux")]
use autopilot_core::conditions::process_condition::ProcessCondition;
use autopilot_core::conditions::{
    Condition, command_condition::CommandCondition, resource_condition::ResourceCondition,
    variable_condition::VariableCondition,
};

#[test]
fn command_condition_checks_exit_code() {
    assert!(CommandCondition::new("echo ok".to_string()).check());
    assert!(!CommandCondition::new("exit 1".to_string()).check());
}

#[test]
fn command_condition_checks_output() {
    let matching = CommandCondition::with_output("echo hello".to_string(), "hello".to_string());
    let mismatched = CommandCondition::with_output("echo hello".to_string(), "world".to_string());

    assert!(matching.check());
    assert!(!mismatched.check());
}

#[test]
fn variable_condition_matches_exact_value() {
    let path = std::env::var("PATH").unwrap_or_default();

    assert!(VariableCondition::new("PATH".to_string(), path).check());
    assert!(!VariableCondition::new("PATH".to_string(), "__not_the_path__".to_string()).check());
}

#[cfg(unix)]
#[test]
fn disk_space_condition_enforces_limits() {
    let has_space = DiskSpaceCondition::new("/".to_string(), 0.0, None);
    let impossible_free = DiskSpaceCondition::new("/".to_string(), f64::MAX, None);
    let no_used_space = DiskSpaceCondition::new("/".to_string(), 0.0, Some(0.0));

    assert!(has_space.check());
    assert!(!impossible_free.check());
    assert!(!no_used_space.check());
}

#[cfg(target_os = "linux")]
#[test]
fn process_condition_finds_current_process() {
    let comm = std::fs::read_to_string("/proc/self/comm")
        .expect("read comm")
        .trim()
        .to_string();

    assert!(ProcessCondition::new(comm, true).check());
    assert!(!ProcessCondition::new("__e2e_missing_process__".to_string(), true).check());
    assert!(ProcessCondition::new("__e2e_missing_process__".to_string(), false).check());
}

#[test]
fn resource_condition_checks_memory_limits() {
    assert!(ResourceCondition::new("memory".to_string(), 101.0, "less".to_string()).check());
    assert!(!ResourceCondition::new("memory".to_string(), 101.0, "greater".to_string()).check());
    assert!(!ResourceCondition::new("unknown".to_string(), 0.0, "less".to_string()).check());
}
