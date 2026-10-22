use tokio::process::Command;

use crate::error::AutoPilotError;

pub fn sync_run(command: String) -> Result<(), AutoPilotError> {
    if let Err(e) = duct_sh::sh_dangerous(&command).run() {
        // error!("Failed to run task '{}': {}", command, e);
        return Err(AutoPilotError::JobExecution(format!(
            "Failed to run task '{}': {}",
            command, e
        )));
    }
    Ok(())
}

pub async fn async_run(command: String) -> Result<(), AutoPilotError> {
    let (shell, flag) = if cfg!(windows) {
        ("cmd", "/C")
    } else {
        ("sh", "-c")
    };

    let status = Command::new(shell)
        .arg(flag)
        .arg(&command)
        .kill_on_drop(true)
        .status()
        .await
        .map_err(|e| {
            AutoPilotError::JobExecution(format!("Failed to run task '{}': {}", command, e))
        })?;

    if !status.success() {
        return Err(AutoPilotError::JobExecution(format!(
            "Failed to run task '{}': exited with {}",
            command, status
        )));
    }

    Ok(())
}
