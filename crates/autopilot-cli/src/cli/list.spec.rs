use super::*;

fn job(id: &str, name: &str) -> Job {
    Job::new(
        id.to_string(),
        name.to_string(),
        String::new(),
        None,
        None,
        vec![],
        vec![],
    )
}

#[test]
fn formats_a_single_job() {
    assert_eq!(format_job(&job("job_1", "Backup")), "job_1 - Backup");
}

#[test]
fn formats_all_jobs_in_order() {
    let jobs = vec![job("job_1", "One"), job("job_2", "Two")];

    assert_eq!(
        format_jobs(&jobs),
        vec!["job_1 - One".to_string(), "job_2 - Two".to_string()]
    );
    assert!(format_jobs(&[]).is_empty());
}
