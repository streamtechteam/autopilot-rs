use super::*;

#[test]
fn responses_serialise_to_expected_json() {
    let status = serde_json::to_value(StatusResponse {
        running: true,
        job_count: 3,
    })
    .unwrap();
    let reload = serde_json::to_value(ReloadResponse {
        success: false,
        message: "Reload failed".to_string(),
    })
    .unwrap();

    assert_eq!(status["running"], true);
    assert_eq!(status["job_count"], 3);
    assert_eq!(reload["success"], false);
    assert_eq!(reload["message"], "Reload failed");
}

#[tokio::test]
async fn app_state_clone_shares_the_underlying_state() {
    let autopilot = AutoPilot::new().await.unwrap();
    let state = AppState {
        auto_pilot: Arc::new(RwLock::new(autopilot)),
        started: Arc::new(AtomicBool::new(false)),
    };
    let clone = state.clone();

    assert!(Arc::ptr_eq(&state.auto_pilot, &clone.auto_pilot));
    assert!(Arc::ptr_eq(&state.started, &clone.started));

    clone
        .started
        .store(true, std::sync::atomic::Ordering::Relaxed);
    assert!(state.started.load(std::sync::atomic::Ordering::Relaxed));

    state.auto_pilot.write().await.shutdown().await.unwrap();
}
