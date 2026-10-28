use tokio::process::Command;

use autopilot_core_error::AutoPilotError;

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
    let status = shell_command(&command)
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

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    // `cmd.exe` does not follow the MSVC argument-escaping rules that `Command`
    // applies to normal arguments, so pass the shell command through verbatim.
    // Otherwise quoted arguments and redirects are corrupted on Windows.
    let mut shell = Command::new("cmd");
    shell.arg("/C").raw_arg(command);
    shell
}

#[cfg(not(windows))]
fn shell_command(command: &str) -> Command {
    let mut shell = Command::new("sh");
    shell.arg("-c").arg(command);
    shell
}

#[cfg(test)]
#[path = "runner.spec.rs"]
mod spec;
