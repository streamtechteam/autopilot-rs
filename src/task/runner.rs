use log::error;

use crate::task::Task;

pub fn sync_run(task: &Task) {
    if let Err(e) = duct_sh::sh_dangerous(task.command.clone()).run() {
        error!("Failed to run task '{}': {}", task.command, e);
    }
}

pub async fn async_run(task: &Task) {
    sync_run(task);
}
