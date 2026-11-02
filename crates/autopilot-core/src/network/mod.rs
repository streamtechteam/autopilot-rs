use std::net::IpAddr;

pub static API_SERVICE_NAME: &str = "autopilot-api";
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: &str = "7883";
/// Must be set to a truthy value before the API can bind to a non-loopback host.
pub const REMOTE_OPT_IN_ENV: &str = "AUTOPILOT_ALLOW_REMOTE";

/// Host the API server binds to. Defaults to loopback and can be overridden
/// with `AUTOPILOT_HOST` when remote access is intentionally enabled.
pub fn host() -> String {
    std::env::var("AUTOPILOT_HOST").unwrap_or_else(|_| DEFAULT_HOST.to_string())
}

/// Whether a bind host only reaches the local machine.
pub fn is_loopback_host(host: &str) -> bool {
    let host = host.trim().trim_start_matches('[').trim_end_matches(']');

    host.eq_ignore_ascii_case("localhost")
        || host
            .parse::<IpAddr>()
            .map(|address| address.is_loopback())
            .unwrap_or(false)
}

/// Explicit opt-in required before binding the API off loopback.
pub fn remote_bind_allowed() -> bool {
    std::env::var(REMOTE_OPT_IN_ENV)
        .map(|value| {
            matches!(
                value.trim().to_ascii_lowercase().as_str(),
                "1" | "true" | "yes"
            )
        })
        .unwrap_or(false)
}

/// Port the API server listens on. Can be overridden with the `AUTOPILOT_PORT`
/// environment variable.
pub fn port() -> String {
    std::env::var("AUTOPILOT_PORT").unwrap_or_else(|_| DEFAULT_PORT.to_string())
}

/// Socket address used by the server, health check, and CLI client.
pub fn address() -> String {
    let host = host();
    let port = port();

    if host.contains(':') && !host.starts_with('[') {
        format!("[{host}]:{port}")
    } else {
        format!("{host}:{port}")
    }
}

#[cfg(test)]
#[path = "network.spec.rs"]
mod spec;
