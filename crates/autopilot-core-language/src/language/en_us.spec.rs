use super::*;

#[test]
fn language_constants_are_populated() {
    let constants = [
        JOB_LOAD_SUCCESS,
        JOB_LOAD_FAILED,
        AUTOPILOT_VERSION,
        AUTOPILOT_SHUTDOWN,
        AUTOPILOT_START,
    ];

    assert!(constants.iter().all(|value| !value.is_empty()));
    assert_eq!(JOB_LOAD_SUCCESS, "Job loaded successfully");
    assert_eq!(AUTOPILOT_START, "AutoPilot started");
}
