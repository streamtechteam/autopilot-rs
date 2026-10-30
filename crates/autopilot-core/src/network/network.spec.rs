use std::sync::Mutex;

use super::*;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn port_uses_default_and_environment_override() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::remove_var("AUTOPILOT_PORT") };
    assert_eq!(port(), DEFAULT_PORT);

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::set_var("AUTOPILOT_PORT", "12345") };
    assert_eq!(port(), "12345");

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::remove_var("AUTOPILOT_PORT") };
}
