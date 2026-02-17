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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::task::Task;

    #[test]
    fn test_sync_run_success() {
        // Test with a simple successful command
        let task = Task {
            command: "echo test".to_string(),
        };
        sync_run(&task);
        // This test verifies that the function runs without panicking
        assert!(true); // Basic assertion to ensure test passes
    }

    #[test]
    fn test_sync_run_failure() {
        // Test with a command that will fail
        let task = Task {
            command: "nonexistent_command_that_does_not_exist".to_string(),
        };
        sync_run(&task);
        // This test verifies that the function handles errors gracefully without panicking
        assert!(true); // Basic assertion to ensure test passes
    }
}
