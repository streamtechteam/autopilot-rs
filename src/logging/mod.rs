use flexi_logger::{Duplicate, FileSpec, Logger, WriteMode};

use crate::directory::get_logs_path;

pub fn init_logging() {
    let _logger = Logger::try_with_str("info, my::critical::module=trace")
        .unwrap()
        .log_to_file(FileSpec::default().directory(get_logs_path()))
        .duplicate_to_stdout(Duplicate::All)
        .write_mode(WriteMode::BufferAndFlush)
        .start()
        .unwrap();

    // #[cfg(target_os = "linux")]
    // {
    //     // init journald or syslog
    //     systemd_journal_logger::JournalLog::new()
    //         .unwrap()
    //         .install().expect("error on installing systemd log");
    //     log::set_max_level(log::LevelFilter::Info);
    // }

    // #[cfg(all(unix, not(target_os = "linux")))]
    // {
    //     // fallback for other unix / macos
    //     let formatter = syslog::Formatter3164 { /* … */ };
    //     let writer = syslog::unix(formatter).unwrap();
    //     let logger = syslog::BasicLogger::new(writer);
    //     log::set_boxed_logger(Box::new(logger)).unwrap();
    //     log::set_max_level(log::LevelFilter::Info);
    // }

    // #[cfg(target_os = "windows")]
    // {
    //     winlog2::register("AutoPilot").unwrap();
    //     winlog2::init("AutoPilot").unwrap();
    //     log::set_max_level(log::LevelFilter::Info);
    // }
}
