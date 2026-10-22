use super::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Mutex;

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

#[test]
fn health_response_round_trips() {
    let json = serde_json::to_string(&HealthResponse {
        status: "ok".to_string(),
        service: API_SERVICE_NAME.to_string(),
    })
    .unwrap();
    let parsed: HealthResponse = serde_json::from_str(&json).unwrap();

    assert_eq!(parsed.status, "ok");
    assert_eq!(parsed.service, "autopilot-api");
}

#[test]
fn detects_a_running_daemon() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let address = listener.local_addr().unwrap();

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::set_var("AUTOPILOT_PORT", address.port().to_string()) };

    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request);
        let body = format!(r#"{{"status":"ok","service":"{}"}}"#, API_SERVICE_NAME);
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        );
        stream.write_all(response.as_bytes()).unwrap();
    });

    assert!(check_if_running());
    server.join().unwrap();

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::remove_var("AUTOPILOT_PORT") };
}

#[test]
fn reports_not_running_when_nothing_listens() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::set_var("AUTOPILOT_PORT", port.to_string()) };
    assert!(!check_if_running());

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::remove_var("AUTOPILOT_PORT") };
}
