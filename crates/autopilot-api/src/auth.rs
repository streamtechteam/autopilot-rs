use axum::{
    extract::{Request, State},
    http::{HeaderValue, StatusCode, header},
    middleware::Next,
    response::Response,
};

use autopilot_core::network::{address, host, port};
use autopilot_core_fs::auth::constant_time_eq;

use crate::state::AppState;

/// Require the bearer token on every route except `/health`.
///
/// Browser-origin requests are rejected unless the `Origin` authority matches
/// the configured server or a loopback alias, and the `Host` header must match
/// when the server is bound to a concrete address.
pub async fn require_auth(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response, StatusCode> {
    if request.uri().path() == "/health" {
        return Ok(next.run(request).await);
    }

    if let Some(origin) = request.headers().get(header::ORIGIN)
        && !origin_allowed(origin)
    {
        return Err(StatusCode::FORBIDDEN);
    }

    if !host_allowed(request.headers().get(header::HOST)) {
        return Err(StatusCode::BAD_REQUEST);
    }

    let provided = request
        .headers()
        .get(header::AUTHORIZATION)
        .and_then(|value| value.to_str().ok())
        .and_then(|value| value.strip_prefix("Bearer "))
        .unwrap_or_default();

    if !constant_time_eq(provided, state.api_token.as_str()) {
        return Err(StatusCode::UNAUTHORIZED);
    }

    Ok(next.run(request).await)
}

fn allowed_authorities() -> Vec<String> {
    let port = port();
    let configured = address();
    let mut authorities = vec![
        configured,
        format!("localhost:{port}"),
        format!("127.0.0.1:{port}"),
        format!("[::1]:{port}"),
    ];
    authorities.sort();
    authorities.dedup();
    authorities
}

fn wildcard_bind() -> bool {
    matches!(host().as_str(), "0.0.0.0" | "::" | "[::]")
}

fn host_allowed(value: Option<&HeaderValue>) -> bool {
    // A wildcard bind accepts whatever address the client used to reach it;
    // the bearer token still has to match.
    if wildcard_bind() {
        return true;
    }

    let Some(value) = value.and_then(|value| value.to_str().ok()) else {
        return false;
    };

    allowed_authorities()
        .iter()
        .any(|authority| authority.eq_ignore_ascii_case(value))
}

fn origin_allowed(value: &HeaderValue) -> bool {
    let Ok(origin) = value.to_str() else {
        return false;
    };
    let Some(authority) = origin
        .strip_prefix("http://")
        .map(|authority| authority.trim_end_matches('/'))
    else {
        return false;
    };

    allowed_authorities()
        .iter()
        .any(|allowed| allowed.eq_ignore_ascii_case(authority))
}

#[cfg(test)]
#[path = "auth.spec.rs"]
mod spec;
