use super::*;
use autopilot_core::task::TaskScheme;

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "autopilot-ui-api-spec-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&path).unwrap();
        Self { path }
    }

    fn api(&self) -> FileJobApi {
        FileJobApi::with_dir(&self.path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.path);
    }
}

fn job(id: &str, name: &str) -> JobScheme {
    JobScheme {
        id: id.to_string(),
        name: Some(name.to_string()),
        description: Some("description".to_string()),
        when: None,
        check_interval: None,
        conditions: Vec::new(),
        tasks: vec![TaskScheme::new("true".to_string())],
    }
}

#[tokio::test]
async fn save_list_and_get_a_job() {
    let dir = TestDir::new();
    let api = dir.api();

    let path = api.save_job(job("job_1", "First")).await.unwrap();

    assert!(path.exists());
    let jobs = api.list_jobs().await.unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, "job_1");

    let loaded = api.get_job("job_1").await.unwrap().unwrap();
    assert_eq!(loaded.name.as_deref(), Some("First"));
    assert!(api.get_job("missing").await.unwrap().is_none());
}

#[tokio::test]
async fn save_job_upserts_by_id() {
    let dir = TestDir::new();
    let api = dir.api();
    let first_path = api.save_job(job("job_1", "First")).await.unwrap();

    let second_path = api.save_job(job("job_1", "Updated")).await.unwrap();

    assert_eq!(first_path, second_path);
    let jobs = api.list_jobs().await.unwrap();
    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].name.as_deref(), Some("Updated"));
}

#[tokio::test]
async fn save_job_removes_duplicate_ids() {
    let dir = TestDir::new();
    let api = dir.api();
    api.save_job(job("job_1", "First")).await.unwrap();
    let duplicate_path = dir.path.join("duplicate.jsonc");
    std::fs::write(
        &duplicate_path,
        serde_json::to_string_pretty(&job("job_1", "Duplicate")).unwrap(),
    )
    .unwrap();

    api.save_job(job("job_1", "Updated")).await.unwrap();

    assert!(!duplicate_path.exists());
    assert_eq!(api.list_jobs().await.unwrap().len(), 1);
}

#[tokio::test]
async fn delete_job_reports_whether_it_existed() {
    let dir = TestDir::new();
    let api = dir.api();
    api.save_job(job("job_1", "First")).await.unwrap();

    assert!(api.delete_job("job_1").await.unwrap());
    assert!(!api.delete_job("job_1").await.unwrap());
    assert!(api.list_jobs().await.unwrap().is_empty());
}

#[tokio::test]
async fn invalid_ids_are_rejected() {
    let dir = TestDir::new();
    let api = dir.api();

    for id in [
        "",
        " ",
        ".",
        "..",
        "nested/job",
        "nested\\job",
        "bad:name",
        "bad*name",
        "trailing ",
        "trailing.",
    ] {
        assert!(api.save_job(job(id, "Invalid")).await.is_err(), "{id}");
        assert!(api.delete_job(id).await.is_err(), "{id}");
    }
}

#[tokio::test]
async fn list_jobs_parses_jsonc_and_skips_invalid_files() {
    let dir = TestDir::new();
    let api = dir.api();
    std::fs::write(
        dir.path.join("commented.jsonc"),
        r#"{
            // comment
            "id": "commented",
            "name": "Commented",
            "description": null,
            "when": null,
            "check_interval": null,
            "conditions": [],
            "tasks": [{"command": "true"}],
        }"#,
    )
    .unwrap();
    std::fs::write(dir.path.join("broken.jsonc"), "{ not json").unwrap();

    let jobs = api.list_jobs().await.unwrap();

    assert_eq!(jobs.len(), 1);
    assert_eq!(jobs[0].id, "commented");
}

#[tokio::test]
async fn new_uses_the_configured_jobs_directory() {
    let dir = TestDir::new();
    autopilot_core::fs::CONFIG_PATH
        .set(dir.path.to_string_lossy().into_owned())
        .ok();

    let api = FileJobApi::new().unwrap();

    assert_eq!(api.jobs_dir(), dir.path.join("jobs"));
}
