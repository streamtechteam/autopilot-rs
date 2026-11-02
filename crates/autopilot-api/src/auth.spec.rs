use super::*;

#[test]
fn browser_origins_must_match_the_server_authority() {
    let _ = host();
    let configured = allowed_authorities();

    assert!(configured.iter().any(|authority| origin_allowed(
        &HeaderValue::from_str(&format!("http://{authority}")).unwrap()
    )));
    assert!(!origin_allowed(&HeaderValue::from_static(
        "http://evil.example"
    )));
    assert!(!origin_allowed(&HeaderValue::from_static(
        "https://localhost:7883"
    )));
    assert!(!origin_allowed(&HeaderValue::from_static("null")));
}
