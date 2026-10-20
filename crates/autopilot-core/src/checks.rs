use std::io::{Read, Write};
use std::net::TcpStream;
pub static PORT: &str = "7883";
pub static API_SERVICE_NAME: &str = "autopilot-api";

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

/// Checks whether the AutoPilot daemon is running by hitting its health endpoint.
pub fn check_if_running() -> bool {
    let mut stream = match TcpStream::connect(format!("localhost:{}", PORT)) {
        Ok(stream) => stream,
        Err(_) => return false,
    };

    let request = format!(
        "GET /health HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n",
        PORT
    );

    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }

    let mut response = String::new();
    if stream.read_to_string(&mut response).is_err() {
        return false;
    }

    let body = match response.split("\r\n\r\n").nth(1) {
        Some(b) => b.trim(),
        None => return false,
    };
    let json_body = serde_json::from_str::<HealthResponse>(body).expect("Failed to parse response");

    json_body.status == "ok" && json_body.service == API_SERVICE_NAME.to_string()
}
