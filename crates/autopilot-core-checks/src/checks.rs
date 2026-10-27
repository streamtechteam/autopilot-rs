use std::{
    io::{Read, Write},
    net::{TcpStream, ToSocketAddrs},
    time::Duration,
};

pub static API_SERVICE_NAME: &str = "autopilot-api";
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: &str = "7883";

const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const IO_TIMEOUT: Duration = Duration::from_secs(2);

/// Host the API server binds to. Defaults to loopback and can be overridden
/// with `AUTOPILOT_HOST` when remote access is intentionally enabled.
pub fn host() -> String {
    std::env::var("AUTOPILOT_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string())
}

/// Port the API server listens on. Can be overridden with the `AUTOPILOT_PORT`
/// environment variable.
pub fn port() -> String {
    std::env::var("AUTOPILOT_PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string())
}

/// Socket address used by the server, health check, and CLI client.
pub fn address() -> String {
    format!("{}:{}", host(), port())
}

#[derive(serde::Serialize, serde::Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

/// Checks whether the AutoPilot daemon is running by hitting its health endpoint.
pub fn check_if_running() -> bool {
    let address = address();
    let Ok(mut addresses) = address.to_socket_addrs() else {
        return false;
    };
    let Some(socket) = addresses.next() else {
        return false;
    };
    let Ok(mut stream) = TcpStream::connect_timeout(&socket, CONNECT_TIMEOUT) else {
        return false;
    };
    if stream.set_read_timeout(Some(IO_TIMEOUT)).is_err()
        || stream.set_write_timeout(Some(IO_TIMEOUT)).is_err()
    {
        return false;
    }

    let request = format!("GET /health HTTP/1.1\r\nHost: {address}\r\nConnection: close\r\n\r\n");

    if stream.write_all(request.as_bytes()).is_err() {
        return false;
    }

    let mut response = String::new();
    if stream.read_to_string(&mut response).is_err() {
        return false;
    }

    let Some(body) = response.split("\r\n\r\n").nth(1) else {
        return false;
    };
    let Ok(json_body) = serde_json::from_str::<HealthResponse>(body.trim()) else {
        return false;
    };

    json_body.status == "ok" && json_body.service == API_SERVICE_NAME
}

#[cfg(test)]
#[path = "checks.spec.rs"]
mod spec;

//  TODO
// refactor this module
