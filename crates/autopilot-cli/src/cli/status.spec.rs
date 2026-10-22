use super::*;

#[test]
fn reports_running_and_stopped_messages() {
    assert_eq!(status_message(true), "Autopilot is running");
    assert_eq!(status_message(false), "Autopilot is not running");
}

#[test]
fn status_handler_does_not_panic() {
    status();
}
