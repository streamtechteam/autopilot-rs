use super::*;

#[test]
fn root_disk_with_no_minimum_passes() {
    let condition = DiskSpaceCondition::new("/".to_string(), 0.0, None);

    assert!(condition.check());
}

#[test]
fn impossible_free_space_and_used_space_fail() {
    assert!(!DiskSpaceCondition::new("/".to_string(), f64::MAX, None).check());
    assert!(!DiskSpaceCondition::new("/".to_string(), 0.0, Some(0.0)).check());
}

#[test]
fn unknown_mount_point_fails() {
    let condition =
        DiskSpaceCondition::new("/autopilot/spec/does/not/exist".to_string(), 0.0, None);

    assert!(!condition.check());
}

#[test]
fn from_scheme_preserves_values_and_reports_name() {
    let condition = DiskSpaceCondition::from_scheme(DiskSpaceConditionScheme {
        path: "/tmp".to_string(),
        min_free_gb: 1.5,
        max_used_gb: Some(100.0),
    });

    assert_eq!(condition.path, "/tmp");
    assert_eq!(condition.min_free_gb, 1.5);
    assert_eq!(condition.max_used_gb, Some(100.0));
    assert_eq!(condition.name(), "Disk Space");
    assert_eq!(condition.clone_box().name(), "Disk Space");
}
