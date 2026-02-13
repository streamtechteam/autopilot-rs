use flexi_logger::{Duplicate, FileSpec, LevelFilter, Logger, WriteMode};

use crate::fs::get_logs_path;

pub fn init_logging(verbose: bool) {
    let log_level = if verbose { "debug" } else { "info" };
    match Logger::try_with_str(log_level)
        .expect("Failed to init logger")
        .log_to_file(FileSpec::default().directory(get_logs_path()))
        .duplicate_to_stdout(Duplicate::All)
        .write_mode(WriteMode::BufferAndFlush)
        .start()
    {
        Ok(_) => {}
        Err(e) => {
            println!("Logger start failed, reason: {}", e);
        }
    };
    let level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    log::set_max_level(level);
    // system logging code (currently is just commented):
    // #[cfg(target_os = "linux")]
    // {
    //     // init journald or syslog
    //     match systemd_journal_logger::JournalLog::new() {
    //         Ok(logger) => match logger.install() {
    //             Ok(_) => {}
    //             Err(e) => {
    //                 println!("Systemd journal logger is not available, reason: {}", e);
    //             }
    //         },
    //         Err(e) => {
    //             println!("Systemd journal logger is not available, reason: {}", e);
    //         }
    //     }
    // }

    // #[cfg(all(unix, not(target_os = "linux")))]
    // {
    //     // fallback for other unix / macos
    //     match syslog::unix(syslog::Formatter3164::default()) {
    //         Ok(writer) => {
    //             let logger = syslog::BasicLogger::new(writer);
    //             let _ = log::set_boxed_logger(Box::new(logger));
    //         }
    //         Err(e) => {
    //             println!("Syslog logger is not available, reason: {}", e);
    //         }
    //     }
    // }

    // #[cfg(target_os = "windows")]
    // {
    //     // Try to register and init winlog2 for Windows Event Log
    //     match winlog2::register("AutoPilot") {
    //         Ok(_) => {
    //             let _ = winlog2::init("AutoPilot");
    //         }
    //         Err(e) => {
    //             println!("Winlog2 logger is not available, reason: {}", e);
    //         }
    //     }
    // }
}
