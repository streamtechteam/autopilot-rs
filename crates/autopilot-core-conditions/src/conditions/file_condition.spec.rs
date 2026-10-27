use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_file(contents: &str) -> std::path::PathBuf {
    let path = std::env::temp_dir().join(format!(
        "autopilot-file-spec-{}-{}.txt",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn exists_checks_file_presence() {
    let path = temp_file("hello");
    let existing = FileCondition::new(
        path.to_string_lossy().into_owned(),
        "EXISTS".into(),
        None,
        None,
    );
    let missing = FileCondition::new(
        path.with_extension("missing")
            .to_string_lossy()
            .into_owned(),
        "exists".into(),
        None,
        None,
    );

    assert!(existing.check());
    assert!(!missing.check());
    assert_eq!(existing.check_type, "exists");

    std::fs::remove_file(path).unwrap();
}

#[test]
fn modified_recently_honours_the_threshold() {
    let path = temp_file("hello");
    let recent = FileCondition::new(
        path.to_string_lossy().into_owned(),
        "modified_recently".into(),
        Some(300),
        None,
    );
    let stale = FileCondition::new(
        path.to_string_lossy().into_owned(),
        "modified_recently".into(),
        Some(-1),
        None,
    );

    assert!(recent.check());
    assert!(!stale.check());

    std::fs::remove_file(path).unwrap();
}

#[test]
fn size_changed_compares_against_the_threshold() {
    let path = temp_file("12345");
    let path = path.to_string_lossy().into_owned();
    let small = FileCondition::new(path.clone(), "size_changed".into(), None, Some(5));
    let large = FileCondition::new(path.clone(), "size_changed".into(), None, Some(6));

    assert!(small.check());
    assert!(!large.check());

    std::fs::remove_file(path).unwrap();
}

#[test]
fn unknown_check_type_and_missing_path_fail() {
    let path = temp_file("hello");
    let unknown = FileCondition::new(
        path.to_string_lossy().into_owned(),
        "unknown".into(),
        None,
        None,
    );

    assert!(!unknown.check());
    assert!(
        !FileCondition::new(
            path.with_extension("missing")
                .to_string_lossy()
                .into_owned(),
            "size_changed".into(),
            None,
            None,
        )
        .check()
    );

    std::fs::remove_file(path).unwrap();
}

#[test]
fn from_scheme_lowercases_check_type() {
    let condition = FileCondition::from_scheme(FileConditionScheme {
        path: "/tmp".to_string(),
        check_type: "EXISTS".to_string(),
        time_threshold: Some(10),
        size_threshold: Some(20),
        check_interval: None,
    });

    assert_eq!(condition.check_type, "exists");
    assert_eq!(condition.time_threshold, Some(10));
    assert_eq!(condition.size_threshold, Some(20));
    assert_eq!(condition.name(), "File");
    assert_eq!(condition.clone_box().name(), "File");
}
