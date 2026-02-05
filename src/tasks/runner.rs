use duct::cmd;
use log::error;

use crate::tasks::Task;

pub fn sync_run(task: &Task) {
    if let Err(e) = cmd("bash", vec!["-c", &task.command]).run() {
        error!("Failed to run task '{}': {}", task.command, e);
    }
}

pub async fn async_run(task: &Task) {
    if let Err(e) = cmd("bash", vec!["-c", &task.command]).run() {
        error!("Failed to run task '{}': {}", task.command, e);
    }
}
