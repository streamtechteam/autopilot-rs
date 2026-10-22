use super::*;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

#[test]
fn stop_fails_when_nothing_is_listening() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::set_var("AUTOPILOT_PORT", port.to_string()) };
    assert!(stop(true).is_err());

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::remove_var("AUTOPILOT_PORT") };
}

#[test]
fn stop_sends_a_shutdown_request() {
    let _guard = ENV_LOCK
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::set_var("AUTOPILOT_PORT", port.to_string()) };

    let server = std::thread::spawn(move || {
        let (mut stream, _) = listener.accept().unwrap();
        let mut request = [0_u8; 1024];
        let read = stream.read(&mut request).unwrap();
        let request = String::from_utf8_lossy(&request[..read]).to_string();
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
            .unwrap();
        request
    });

    assert!(stop(true).is_ok());
    let request = server.join().unwrap();
    assert!(request.starts_with("GET /shutdown HTTP/1.1"));

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::remove_var("AUTOPILOT_PORT") };
}
