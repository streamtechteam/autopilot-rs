use super::*;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use tokio::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::const_new(());

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn starts_an_http_server_with_the_health_route() {
    let _guard = ENV_LOCK.lock().await;
    let probe = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = probe.local_addr().unwrap().port();
    drop(probe);

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::set_var("AUTOPILOT_PORT", port.to_string()) };

    let autopilot = autopilot_core::autopilot::AutoPilot::new().await.unwrap();
    let state = AppState {
        auto_pilot: std::sync::Arc::new(tokio::sync::RwLock::new(autopilot)),
    };
    let handle = start_api(state.clone()).await.unwrap();

    let mut stream = None;
    for _ in 0..100 {
        match TcpStream::connect(("127.0.0.1", port)) {
            Ok(connected) => {
                stream = Some(connected);
                break;
            }
            Err(_) => tokio::time::sleep(std::time::Duration::from_millis(10)).await,
        }
    }
    let mut stream = stream.expect("API server did not start listening");
    stream
        .write_all(
            format!("GET /health HTTP/1.1\r\nHost: localhost:{port}\r\nConnection: close\r\n\r\n")
                .as_bytes(),
        )
        .unwrap();
    stream
        .set_read_timeout(Some(std::time::Duration::from_millis(250)))
        .unwrap();
    let mut response = Vec::new();
    let mut buffer = [0_u8; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => response.extend_from_slice(&buffer[..read]),
            Err(error)
                if matches!(
                    error.kind(),
                    std::io::ErrorKind::WouldBlock | std::io::ErrorKind::TimedOut
                ) =>
            {
                break;
            }
            Err(error) => panic!("failed to read response: {error}"),
        }
    }
    drop(stream);
    let response = String::from_utf8(response).unwrap();

    assert!(response.starts_with("HTTP/1.1 200"));
    assert!(response.contains(r#""status":"ok""#));
    assert!(response.contains(r#""service":"autopilot-api""#));

    handle.abort();
    state.auto_pilot.write().await.shutdown().await.unwrap();

    // SAFETY: env access is serialised by ENV_LOCK.
    unsafe { std::env::remove_var("AUTOPILOT_PORT") };
}
