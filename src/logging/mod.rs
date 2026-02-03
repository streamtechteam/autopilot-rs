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

    #[cfg(target_os = "linux")]
    {
        // init journald or syslog
        if let Ok(logger) = systemd_journal_logger::JournalLog::new() {
            let _ = logger.install();
            log::set_max_level(log::LevelFilter::Info);
        }
    }

    #[cfg(all(unix, not(target_os = "linux")))]
    {
        // fallback for other unix / macos
        if let Ok(writer) = syslog::unix(syslog::Formatter3164::default()) {
            let logger = syslog::BasicLogger::new(writer);
            let _ = log::set_boxed_logger(Box::new(logger));
            log::set_max_level(log::LevelFilter::Info);
        }
    }

    #[cfg(target_os = "windows")]
    {
        // On Windows, we'll stick to flexi_logger's file logging for now
        // as winlog2 requires more setup and can be brittle
        log::set_max_level(log::LevelFilter::Info);
    }
}
