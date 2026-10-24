use super::*;

#[test]
fn initialises_a_file_logger_at_info_level() {
    let dir = std::env::temp_dir().join(format!(
        "autopilot-core-fs-logs-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();

    init_logger(false, &dir);

    assert_eq!(log::max_level(), log::LevelFilter::Info);
}
