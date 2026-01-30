use duct::cmd;

use crate::tasks::Task;

pub fn sync_run(task: &Task) {
    cmd("bash", vec!["-c", &task.command]).run();
}

pub async fn async_run(task: &Task) {
    cmd("bash", vec!["-c", &task.command]).run();
}
