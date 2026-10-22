use super::*;
use crate::fs::test_support;

#[test]
fn creates_all_required_paths() {
    let _guard = test_support::lock();

    set_all_paths(true).unwrap();

    assert!(std::path::Path::new(&get_autopilot_path()).is_dir());
    assert!(std::path::Path::new(&get_logs_path()).is_dir());
    assert!(std::path::Path::new(&get_jobs_path()).is_dir());
    assert!(std::path::Path::new(&get_config_path()).is_file());
    assert_eq!(std::fs::read_to_string(get_config_path()).unwrap(), "{}");
}

#[test]
fn individual_setters_create_directories() {
    let _guard = test_support::lock();

    set_logs_path().unwrap();
    set_jobs_path().unwrap();
    set_config_path(true).unwrap();

    assert!(std::path::Path::new(&get_logs_path()).is_dir());
    assert!(std::path::Path::new(&get_jobs_path()).is_dir());
    assert!(std::path::Path::new(&get_config_path()).is_file());
}

#[test]
fn setting_paths_is_idempotent() {
    let _guard = test_support::lock();

    set_all_paths(true).unwrap();
    set_all_paths(true).unwrap();

    assert!(std::path::Path::new(&get_autopilot_path()).is_dir());
}
