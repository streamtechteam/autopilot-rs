use super::*;

#[test]
fn sync_run_executes_successful_commands() {
    assert!(sync_run("true".to_string()).is_ok());
    assert!(sync_run("echo autopilot".to_string()).is_ok());
}

#[test]
fn sync_run_reports_failures() {
    let error = sync_run("exit 3".to_string()).unwrap_err();

    assert!(matches!(error, AutoPilotError::JobExecution(_)));
    assert!(error.to_string().contains("exit 3"));
}

#[tokio::test]
async fn async_run_executes_successful_commands() {
    assert!(async_run("true".to_string(), None).await.is_ok());
}

#[tokio::test]
async fn async_run_reports_failures() {
    let error = async_run("exit 9".to_string(), None).await.unwrap_err();

    assert!(matches!(error, AutoPilotError::JobExecution(_)));
    assert!(error.to_string().contains("exit 9"));
}

#[cfg(unix)]
#[tokio::test]
async fn async_run_times_out_and_kills_the_command() {
    let error = async_run("sleep 5".to_string(), Some(1)).await.unwrap_err();

    assert!(matches!(error, AutoPilotError::JobExecution(_)));
    assert!(error.to_string().contains("timed out after 1 seconds"));
}
