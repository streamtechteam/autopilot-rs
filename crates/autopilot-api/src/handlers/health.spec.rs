use super::{API_SERVICE_NAME, health};

#[tokio::test]
async fn health_reports_ok() {
    let response = health().await.0;

    assert_eq!(response.status, "ok");
    assert_eq!(response.service, API_SERVICE_NAME);
}
