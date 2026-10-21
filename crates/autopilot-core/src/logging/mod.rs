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
    // TODO: system logging (journald/syslog/Windows Event Log)
}
