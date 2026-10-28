use super::*;
use crate::job::TaskMode;
use crate::job::get::get_job;
use crate::job::test_support;
use autopilot_core_task::TaskScheme;

fn add(name: Option<&str>) -> PathBuf {
    add_job(
        name.map(str::to_string),
        Some("spec job".to_string()),
        None,
        None,
        vec![ConditionScheme::Fail(
            autopilot_core_conditions::fail_condition::FailCondition,
        )],
        vec![TaskScheme::new("true".to_string())],
        TaskMode::Concurrent,
    )
    .unwrap()
}

#[test]
fn add_job_writes_a_jsonc_file() {
    let _guard = test_support::lock();
    let path = add(Some("named"));

    assert!(path.exists());
    assert!(path.to_string_lossy().ends_with(".jsonc"));

    let job = get_job(path.clone()).unwrap();
    assert_eq!(path.file_stem().unwrap(), job.id.as_str());
    assert_eq!(job.name, "named");
    assert_eq!(job.description, "spec job");

    std::fs::remove_file(path).unwrap();
}

#[test]
fn add_job_without_name_uses_the_generated_id() {
    let _guard = test_support::lock();
    let path = add(None);

    assert!(path.exists());
    assert!(path.to_string_lossy().contains("job_"));

    std::fs::remove_file(path).unwrap();
}

#[test]
fn update_job_preserves_the_id_and_path() {
    let _guard = test_support::lock();
    let path = add(Some("before"));
    let job = get_job(path.clone()).unwrap();

    let updated_path = update_job(
        &job.id,
        Some("after".to_string()),
        Some("updated".to_string()),
        None,
        None,
        vec![],
        vec![],
        Some(TaskMode::Sequential),
    )
    .unwrap();

    assert_eq!(updated_path, path);
    assert!(path.exists());

    let updated = get_job(path.clone()).unwrap();
    assert_eq!(updated.id, job.id);
    assert_eq!(updated.name, "after");
    assert_eq!(updated.description, "updated");
    assert_eq!(updated.task_mode, TaskMode::Sequential);

    update_job(
        &updated.id,
        Some("after again".to_string()),
        None,
        None,
        None,
        vec![],
        vec![],
        None,
    )
    .unwrap();

    let preserved = get_job(path.clone()).unwrap();
    assert_eq!(preserved.task_mode, TaskMode::Sequential);

    std::fs::remove_file(path).unwrap();
}

#[test]
fn update_job_reports_unknown_ids() {
    let _guard = test_support::lock();
    let error =
        update_job("job_missing", None, None, None, None, vec![], vec![], None).unwrap_err();
    assert!(error.to_string().contains("not found"));
}

#[test]
fn remove_job_requires_an_identifier() {
    let error = remove_job(None, None).unwrap_err();

    assert!(error.to_string().contains("No job ID or file name"));
}

#[test]
fn remove_job_by_file_name() {
    let _guard = test_support::lock();
    let path = add(Some("by_file"));
    let file_name = path.file_name().unwrap().to_string_lossy().into_owned();

    remove_job(None, Some(file_name)).unwrap();
    assert!(!path.exists());
}

#[test]
fn remove_job_rejects_path_traversal() {
    let _guard = test_support::lock();

    let error = remove_job(None, Some("../outside.jsonc".to_string())).unwrap_err();
    assert!(error.to_string().contains("path separators"));
}

#[test]
fn remove_job_by_file_name_without_extension() {
    let _guard = test_support::lock();
    let path = add(Some("without_extension"));
    let without_extension = path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .trim_end_matches(".jsonc")
        .to_string();

    remove_job(None, Some(without_extension)).unwrap();
    assert!(!path.exists());
}

#[test]
fn remove_job_by_id() {
    let _guard = test_support::lock();
    let path = add(Some("by_id"));
    let job = get_job(path.clone()).unwrap();

    remove_job(Some(job.id), None).unwrap();
    assert!(!path.exists());
}

#[test]
fn remove_job_reports_unknown_ids() {
    let _guard = test_support::lock();
    let error = remove_job(Some("job_that_does_not_exist".to_string()), None).unwrap_err();

    assert!(error.to_string().contains("not found"));
}
