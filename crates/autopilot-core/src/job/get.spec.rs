use super::*;
use crate::fs::test_support;

fn write_job(file_name: &str, _id: &str, contents: &str) -> PathBuf {
    let path = test_support::jobs_dir().join(file_name);
    std::fs::write(&path, contents).unwrap();
    path
}

#[test]
fn parses_plain_json_and_jsonc() {
    let plain = parse_job(r#"{"id":"plain","conditions":[],"tasks":[]}"#).unwrap();
    let commented = parse_job(
        r#"{
            // a comment
            "id": "commented",
            "conditions": [],
            "tasks": [],
        }"#,
    )
    .unwrap();

    assert_eq!(plain.id, "plain");
    assert_eq!(commented.id, "commented");
    assert!(parse_job("not json").is_err());
}

#[test]
fn get_jobs_reads_valid_files_and_skips_invalid_ones() {
    let _guard = test_support::lock();
    let valid = write_job(
        "job_get_valid.jsonc",
        "job_get_valid",
        r#"{"id":"job_get_valid","conditions":[],"tasks":[]}"#,
    );
    let commented = write_job(
        "job_get_commented.jsonc",
        "job_get_commented",
        "{\n// comment\n\"id\":\"job_get_commented\",\"conditions\":[],\"tasks\":[],\n}",
    );
    let invalid = write_job("job_get_invalid.jsonc", "ignored", "not json");

    let jobs = get_jobs(true);
    let ids: Vec<&str> = jobs.iter().map(|job| job.id.as_str()).collect();

    assert!(ids.contains(&"job_get_valid"));
    assert!(ids.contains(&"job_get_commented"));
    assert!(!ids.contains(&"ignored"));

    for path in [valid, commented, invalid] {
        std::fs::remove_file(path).unwrap();
    }
}

#[test]
fn get_jobs_paths_lists_only_files() {
    let _guard = test_support::lock();
    let file = write_job(
        "job_get_paths.jsonc",
        "job_get_paths",
        r#"{"id":"job_get_paths","conditions":[],"tasks":[]}"#,
    );
    let directory = test_support::jobs_dir().join("job_get_paths_dir");
    std::fs::create_dir_all(&directory).unwrap();

    let paths = get_jobs_paths();

    assert!(paths.contains(&file));
    assert!(!paths.contains(&directory));

    std::fs::remove_file(file).unwrap();
    std::fs::remove_dir(directory).unwrap();
}

#[test]
fn get_job_reads_a_single_file() {
    let _guard = test_support::lock();
    let path = write_job(
        "job_get_single.jsonc",
        "job_get_single",
        r#"{"id":"job_get_single","conditions":[],"tasks":[]}"#,
    );

    assert_eq!(get_job(path.clone()).unwrap().id, "job_get_single");
    assert!(get_job(test_support::jobs_dir().join("missing.jsonc")).is_err());

    std::fs::write(&path, "not json").unwrap();
    assert!(get_job(path.clone()).is_err());

    std::fs::remove_file(path).unwrap();
}
