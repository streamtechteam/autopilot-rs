use super::*;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::const_new(());

fn free_port() -> u16 {
    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    probe.local_addr().unwrap().port()
}

fn request(port: u16, method: &str, path: &str, headers: &[(&str, &str)]) -> String {
    let mut stream = TcpStream::connect(("127.0.0.1", port)).expect("connect");
    let mut request = format!("{method} {path} HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\n");

    for (name, value) in headers {
        if name.eq_ignore_ascii_case("host") {
            // Replace the default Host header.
            request = format!("{method} {path} HTTP/1.1\r\n");
            request.push_str(&format!("{name}: {value}\r\n"));
        } else {
            request.push_str(&format!("{name}: {value}\r\n"));
        }
    }

    request.push_str("Connection: close\r\n\r\n");
    stream.write_all(request.as_bytes()).unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_secs(2)))
        .unwrap();

    let mut response = String::new();
    stream.read_to_string(&mut response).unwrap();
    response
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn requires_a_bearer_token_and_rejects_cross_origin_requests() {
    let _guard = ENV_LOCK.lock().await;
    let port = free_port();

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe {
        std::env::set_var("AUTOPILOT_PORT", port.to_string());
        std::env::set_var("AUTOPILOT_API_TOKEN", "test-token");
        std::env::remove_var("AUTOPILOT_HOST");
    }

    let autopilot = autopilot_core::autopilot::AutoPilot::new().await.unwrap();
    let state = AppState::new(std::sync::Arc::new(tokio::sync::RwLock::new(autopilot)));
    let handle = start_api(state.clone()).await.unwrap();

    let auth = ("Authorization", "Bearer test-token");

    // Health stays reachable without credentials.
    let health = request(port, "GET", "/health", &[]);
    assert!(health.starts_with("HTTP/1.1 200"));
    assert!(health.contains(r#""status":"ok""#));
    assert!(health.contains(r#""service":"autopilot-api""#));

    assert!(request(port, "GET", "/status", &[]).starts_with("HTTP/1.1 401"));
    assert!(
        request(port, "GET", "/status", &[("Authorization", "Bearer wrong")])
            .starts_with("HTTP/1.1 401")
    );
    assert!(
        request(port, "POST", "/shutdown", &[]).starts_with("HTTP/1.1 401"),
        "unauthenticated shutdown must be rejected"
    );

    let status = request(port, "GET", "/status", &[auth]);
    assert!(status.starts_with("HTTP/1.1 200"), "{status}");
    assert!(status.contains(r#""running":false"#));

    assert!(
        request(
            port,
            "GET",
            "/status",
            &[auth, ("Origin", "http://evil.example")]
        )
        .starts_with("HTTP/1.1 403")
    );
    assert!(
        request(port, "GET", "/status", &[auth, ("Host", "evil.example")])
            .starts_with("HTTP/1.1 400")
    );

    handle.abort();
    state.auto_pilot.write().await.shutdown().await.unwrap();

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe {
        std::env::remove_var("AUTOPILOT_PORT");
        std::env::remove_var("AUTOPILOT_API_TOKEN");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn refuses_non_loopback_binds_without_an_explicit_opt_in() {
    let _guard = ENV_LOCK.lock().await;

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe {
        std::env::set_var("AUTOPILOT_HOST", "0.0.0.0");
        std::env::set_var("AUTOPILOT_PORT", free_port().to_string());
        std::env::set_var("AUTOPILOT_API_TOKEN", "test-token");
        std::env::remove_var("AUTOPILOT_ALLOW_REMOTE");
    }

    let autopilot = autopilot_core::autopilot::AutoPilot::new().await.unwrap();
    let state = AppState::new(std::sync::Arc::new(tokio::sync::RwLock::new(autopilot)));
    let error = start_api(state).await.unwrap_err();

    assert!(
        error.to_string().contains("non-loopback"),
        "unexpected error: {error}"
    );
    assert!(error.to_string().contains("AUTOPILOT_ALLOW_REMOTE"));

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe {
        std::env::remove_var("AUTOPILOT_HOST");
        std::env::remove_var("AUTOPILOT_PORT");
        std::env::remove_var("AUTOPILOT_API_TOKEN");
    }
}
