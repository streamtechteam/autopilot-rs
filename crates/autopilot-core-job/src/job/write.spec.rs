use super::*;
use crate::job::TaskMode;
use crate::job::read::get;
use crate::job::test_support;
use autopilot_core_task::TaskScheme;

fn make_job(name: Option<&str>) -> PathBuf {
    add(
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
fn add_writes_a_jsonc_file() {
    let _guard = test_support::lock();
    let path = make_job(Some("named"));

    assert!(path.exists());
    assert!(path.to_string_lossy().ends_with(".jsonc"));

    let job = get(path.clone()).unwrap();
    assert_eq!(path.file_stem().unwrap(), job.id.as_str());
    assert_eq!(job.name, "named");
    assert_eq!(job.description, "spec job");

    std::fs::remove_file(path).unwrap();
}

#[test]
fn add_without_name_uses_the_generated_id() {
    let _guard = test_support::lock();
    let path = make_job(None);

    assert!(path.exists());
    assert!(path.to_string_lossy().contains("job_"));

    std::fs::remove_file(path).unwrap();
}

#[test]
fn update_preserves_the_id_and_path() {
    let _guard = test_support::lock();
    let path = make_job(Some("before"));
    let job = get(path.clone()).unwrap();

    let updated_path = update(
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

    let updated = get(path.clone()).unwrap();
    assert_eq!(updated.id, job.id);
    assert_eq!(updated.name, "after");
    assert_eq!(updated.description, "updated");
    assert_eq!(updated.task_mode, TaskMode::Sequential);

    update(
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

    let preserved = get(path.clone()).unwrap();
    assert_eq!(preserved.task_mode, TaskMode::Sequential);

    std::fs::remove_file(path).unwrap();
}

#[test]
fn update_reports_unknown_ids() {
    let _guard = test_support::lock();
    let error = update("job_missing", None, None, None, None, vec![], vec![], None).unwrap_err();
    assert!(error.to_string().contains("not found"));
}

#[test]
fn remove_requires_an_identifier() {
    let error = remove(None, None).unwrap_err();

    assert!(error.to_string().contains("No job ID or file name"));
}

#[test]
fn remove_by_file_name() {
    let _guard = test_support::lock();
    let path = make_job(Some("by_file"));
    let file_name = path.file_name().unwrap().to_string_lossy().into_owned();

    remove(None, Some(file_name)).unwrap();
    assert!(!path.exists());
}

#[test]
fn remove_rejects_path_traversal() {
    let _guard = test_support::lock();

    let error = remove(None, Some("../outside.jsonc".to_string())).unwrap_err();
    assert!(error.to_string().contains("path separators"));
}

#[test]
fn remove_by_file_name_without_extension() {
    let _guard = test_support::lock();
    let path = make_job(Some("without_extension"));
    let without_extension = path
        .file_name()
        .unwrap()
        .to_string_lossy()
        .trim_end_matches(".jsonc")
        .to_string();

    remove(None, Some(without_extension)).unwrap();
    assert!(!path.exists());
}

#[test]
fn remove_by_id() {
    let _guard = test_support::lock();
    let path = make_job(Some("by_id"));
    let job = get(path.clone()).unwrap();

    remove(Some(job.id), None).unwrap();
    assert!(!path.exists());
}

#[test]
fn remove_reports_unknown_ids() {
    let _guard = test_support::lock();
    let error = remove(Some("job_that_does_not_exist".to_string()), None).unwrap_err();

    assert!(error.to_string().contains("not found"));
}
