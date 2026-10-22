use super::*;
use crate::fs::test_support;
use crate::task::TaskScheme;

fn add(name: Option<&str>) -> PathBuf {
    add_job(
        name.map(str::to_string),
        Some("spec job".to_string()),
        None,
        None,
        vec![ConditionScheme::Fail(
            crate::conditions::fail_condition::FailCondition,
        )],
        vec![TaskScheme::new("true".to_string())],
    )
    .unwrap()
}

#[test]
fn add_job_writes_a_jsonc_file() {
    let _guard = test_support::lock();
    let path = add(Some("named"));

    assert!(path.exists());
    assert!(path.to_string_lossy().ends_with(".jsonc"));
    assert!(path.to_string_lossy().contains("named"));

    let job = get_job(path.clone()).unwrap();
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
fn remove_job_requires_an_identifier() {
    let error = remove_job(None, None).unwrap_err();

    assert!(error.to_string().contains("No job ID or file name"));
}

#[test]
fn remove_job_by_absolute_file_name() {
    let _guard = test_support::lock();
    let path = add(Some("by_file"));

    remove_job(None, Some(path.to_string_lossy().into_owned())).unwrap();
    assert!(!path.exists());
}

#[test]
fn remove_job_by_file_name_without_extension() {
    let _guard = test_support::lock();
    let path = add(Some("without_extension"));
    let without_extension = path
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
