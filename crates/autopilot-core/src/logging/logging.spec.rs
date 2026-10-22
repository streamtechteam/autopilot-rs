use super::*;
use crate::fs::test_support;

#[test]
fn initialises_a_file_logger_at_info_level() {
    let _guard = test_support::lock();
    test_support::config_dir();

    init_logging(false);

    assert_eq!(log::max_level(), log::LevelFilter::Info);
}
