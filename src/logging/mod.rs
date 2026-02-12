use flexi_logger::{Duplicate, FileSpec, LevelFilter, Logger, WriteMode};

use crate::fs::get_logs_path;

pub fn init_logging(verbose: bool) {
    let log_level = if verbose { "debug" } else { "info" };
    
    let _logger = Logger::try_with_str(log_level)
        .expect("Failed to init logger")
        .log_to_file(FileSpec::default().directory(get_logs_path()))
        .duplicate_to_stdout(Duplicate::All)
        .write_mode(WriteMode::BufferAndFlush)
        .start()
        .expect("Failed to start logger");

    #[cfg(target_os = "linux")]
    {
        // init journald or syslog
        if let Ok(logger) = systemd_journal_logger::JournalLog::new() {
            let _ = logger.install();
            let level = if verbose {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            };
            log::set_max_level(level);
        }
    }

    #[cfg(all(unix, not(target_os = "linux")))]
    {
        // fallback for other unix / macos
        if let Ok(writer) = syslog::unix(syslog::Formatter3164::default()) {
            let logger = syslog::BasicLogger::new(writer);
            let _ = log::set_boxed_logger(Box::new(logger));
            let level = if verbose {
                log::LevelFilter::Debug
            } else {
                log::LevelFilter::Info
            };
            log::set_max_level(level);
        }
    }

    #[cfg(target_os = "windows")]
    {
        // Try to register and init winlog2 for Windows Event Log
        if let Ok(_) = winlog2::register("AutoPilot") {
            let _ = winlog2::init("AutoPilot");
        }
        let level = if verbose {
            log::LevelFilter::Debug
        } else {
            log::LevelFilter::Info
        };
        log::set_max_level(level);
    }
}
