use super::*;

#[test]
fn constructs_and_serialises_tasks() {
    let task = Task::new("echo hello".to_string());
    let json = serde_json::to_value(&task).unwrap();

    assert_eq!(task.command, "echo hello");
    assert_eq!(json["command"], "echo hello");
    assert_eq!(
        serde_json::from_value::<Task>(json).unwrap().command,
        "echo hello"
    );
    assert!(
        serde_json::from_value::<Task>(serde_json::json!({"command": "true", "extra": 1})).is_err()
    );
}

#[tokio::test]
async fn run_returns_a_join_handle_for_success() {
    let task = Task::new("true".to_string());

    assert!(task.run().await.unwrap().is_ok());
}

#[tokio::test]
async fn run_propagates_command_failures() {
    let task = Task::new("exit 7".to_string());

    assert!(task.run().await.unwrap().is_err());
}

#[test]
fn run_sync_succeeds_and_fails() {
    assert!(Task::new("true".to_string()).run_sync().is_ok());
    let error = Task::new("exit 7".to_string()).run_sync().unwrap_err();

    assert!(error.to_string().contains("Failed to run task"));
}
