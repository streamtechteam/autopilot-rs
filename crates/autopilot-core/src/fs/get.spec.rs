use super::*;
use crate::fs::test_support;

#[test]
fn builds_paths_relative_to_the_config_directory() {
    let _guard = test_support::lock();
    let root = test_support::config_dir();

    assert_eq!(get_autopilot_path(), root);
    assert_eq!(get_logs_path(), format!("{root}/logs"));
    assert_eq!(get_config_path(), format!("{root}/autopilot.jsonc"));
    assert_eq!(get_jobs_path(), format!("{root}/jobs"));
    assert_eq!(
        get_jobs_dir().unwrap(),
        std::path::PathBuf::from(format!("{root}/jobs"))
    );
}
