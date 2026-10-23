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
    assert!(async_run("true".to_string()).await.is_ok());
}

#[tokio::test]
async fn async_run_reports_failures() {
    let error = async_run("exit 9".to_string()).await.unwrap_err();

    assert!(matches!(error, AutoPilotError::JobExecution(_)));
    assert!(error.to_string().contains("exit 9"));
}
