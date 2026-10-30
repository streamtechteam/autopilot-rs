pub static API_SERVICE_NAME: &str = "autopilot-api";
pub const DEFAULT_HOST: &str = "127.0.0.1";
pub const DEFAULT_PORT: &str = "7883";

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

#[cfg(test)]
#[path = "network.spec.rs"]
mod spec;
