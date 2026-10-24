use std::path::{Path, PathBuf};

use autopilot_core_fs::paths;
use flexi_logger::{Duplicate, FileSpec, LevelFilter, Logger, WriteMode};

pub fn init_logging(verbose: bool) {
    let logs_dir = paths()
        .map(|paths| paths.logs_dir())
        .unwrap_or_else(|_| PathBuf::from("logs"));
    init_logger(verbose, &logs_dir);
}

pub fn init_logger(verbose: bool, logs_dir: impl AsRef<Path>) {
    let logs_dir = logs_dir.as_ref().to_path_buf();
    let log_level = if verbose { "debug" } else { "info" };
    match Logger::try_with_str(&log_level)
        .expect("Failed to init logger")
        .log_to_file(FileSpec::default().directory(&logs_dir))
        .duplicate_to_stdout(Duplicate::All)
        .write_mode(WriteMode::BufferAndFlush)
        .start()
    {
        Ok(_) => {}
        Err(error) => {
            println!("Logger start failed, reason: {error}");
        }
    };
    let level = if verbose {
        LevelFilter::Debug
    } else {
        LevelFilter::Info
    };
    log::set_max_level(level);
}

#[cfg(test)]
#[path = "logging.spec.rs"]
mod spec;
