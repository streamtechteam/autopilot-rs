use std::time::Duration;

use autopilot_core::task::Task;

#[tokio::test]
async fn failing_task_returns_error() {
    let result = Task::new("exit 7".to_string()).run().await;

    let error = result
        .expect("task join handle")
        .expect_err("exit 7 should fail");
    assert!(error.to_string().contains("exit"));
}

#[cfg(unix)]
#[tokio::test]
async fn aborting_task_prevents_completion() {
    let marker = std::env::temp_dir().join(format!(
        "autopilot-e2e-abort-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let task = Task::new(format!("sleep 1 && touch {}", marker.display()));

    let handle = task.run();
    tokio::time::sleep(Duration::from_millis(100)).await;
    handle.abort();
    tokio::time::sleep(Duration::from_millis(1200)).await;

    assert!(!marker.exists(), "aborted task still created its marker");
}
