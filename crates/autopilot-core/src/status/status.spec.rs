use super::*;

#[tokio::test]
async fn stores_new_statuses_and_lookup_by_id() {
    let manager = StateManager::new();

    manager
        .set(
            "job_1".to_string(),
            Some("First".to_string()),
            JobStatusEnum::Running,
        )
        .await;

    assert_eq!(
        manager.get_state_by_id("job_1".to_string()).await,
        Some(JobStatusEnum::Running)
    );
    let stored = manager
        .get_state_struct_by_id("job_1".to_string())
        .await
        .unwrap();
    assert_eq!(stored.name, "First");
    assert!(matches!(stored.status, JobStatusEnum::Running));
    assert_eq!(manager.get_all().await.len(), 1);
}

#[tokio::test]
async fn updates_the_existing_entry_without_changing_its_position() {
    let manager = StateManager::new();
    manager
        .set(
            "job_1".to_string(),
            Some("First".to_string()),
            JobStatusEnum::Pending,
        )
        .await;
    manager
        .set(
            "job_2".to_string(),
            Some("Second".to_string()),
            JobStatusEnum::Unknown,
        )
        .await;

    manager
        .set("job_1".to_string(), None, JobStatusEnum::Completed)
        .await;

    let all = manager.get_all().await;
    assert_eq!(all.len(), 2);
    assert_eq!(all[0].id, "job_1");
    assert!(matches!(all[0].status, JobStatusEnum::Completed));
    assert_eq!(all[0].name, "First");
}

#[tokio::test]
async fn get_status_log_contains_time_and_statuses() {
    let manager = StateManager::new();
    manager
        .set(
            "job_1".to_string(),
            Some("First".to_string()),
            JobStatusEnum::Waiting,
        )
        .await;

    let log = manager.get_status_log().await;
    assert!(!log.time.is_empty());
    assert_eq!(log.statuses.len(), 1);
    assert_eq!(log.statuses[0].name, "First");
}

#[tokio::test]
async fn missing_ids_return_none() {
    let manager = StateManager::new();

    assert!(
        manager
            .get_state_by_id("missing".to_string())
            .await
            .is_none()
    );
    assert!(
        manager
            .get_state_struct_by_id("missing".to_string())
            .await
            .is_none()
    );
}

#[tokio::test]
async fn clones_share_the_same_state() {
    let manager = StateManager::new();
    let clone = manager.clone();

    manager
        .set(
            "job_1".to_string(),
            Some("First".to_string()),
            JobStatusEnum::Scheduled,
        )
        .await;

    assert_eq!(
        clone.get_state_by_id("job_1".to_string()).await,
        Some(JobStatusEnum::Scheduled)
    );
}

#[test]
fn status_struct_and_enum_serialise() {
    let status = JobStatusStruct::new(
        "job_1".to_string(),
        "First".to_string(),
        JobStatusEnum::Failed,
    );
    let json = serde_json::to_value(&status).unwrap();

    assert_eq!(json["id"], "job_1");
    assert_eq!(json["name"], "First");
    assert_eq!(json["status"], "Failed");
    assert_eq!(
        serde_json::to_string(&JobStatusEnum::Cancelled).unwrap(),
        "\"Cancelled\""
    );
}
