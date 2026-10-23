use super::*;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_dir() -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "autopilot-search-spec-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(dir.join("nested")).unwrap();
    dir
}

#[test]
fn searches_files_recursively() {
    let dir = temp_dir();
    std::fs::write(dir.join("one.jsonc"), "{\"id\":\"target\"}").unwrap();
    std::fs::write(dir.join("nested/two.jsonc"), "{\"id\":\"other\"}").unwrap();

    assert!(search_directory(&dir, "target").is_ok());

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn rejects_invalid_regular_expressions() {
    let dir = temp_dir();
    let error = search_directory(&dir, "[").unwrap_err();

    assert_eq!(error.kind(), std::io::ErrorKind::InvalidInput);

    std::fs::remove_dir_all(dir).unwrap();
}

#[test]
fn missing_directories_are_tolerated() {
    let dir = temp_dir().join("missing");

    assert!(search_directory(&dir, "anything").is_ok());
}
