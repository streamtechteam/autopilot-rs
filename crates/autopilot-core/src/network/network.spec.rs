use super::*;
use crate::test_support::ENV_LOCK;

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

#[test]
fn loopback_hosts_are_recognised() {
    assert!(is_loopback_host("127.0.0.1"));
    assert!(is_loopback_host("127.0.0.42"));
    assert!(is_loopback_host("::1"));
    assert!(is_loopback_host("[::1]"));
    assert!(is_loopback_host("localhost"));
    assert!(is_loopback_host("LOCALHOST"));

    assert!(!is_loopback_host("0.0.0.0"));
    assert!(!is_loopback_host("192.168.1.10"));
    assert!(!is_loopback_host("example.com"));
}

#[test]
fn port_and_ipv6_addresses_are_formatted_for_clients() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::set_var("AUTOPILOT_HOST", "::1") };
    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::set_var("AUTOPILOT_PORT", "12345") };
    assert_eq!(address(), "[::1]:12345");

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::remove_var("AUTOPILOT_HOST") };
    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::remove_var("AUTOPILOT_PORT") };
}
