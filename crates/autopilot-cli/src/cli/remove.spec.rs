use super::*;

#[test]
fn formats_known_job_options() {
    let path = std::path::Path::new("/tmp/job_123_Backup.jsonc");

    assert_eq!(
        format_job_option(Some("Backup"), path),
        "Backup - job_123_Backup.jsonc"
    );
}

#[test]
fn formats_unknown_job_options() {
    let path = std::path::Path::new("/tmp/broken.jsonc");

    assert_eq!(format_job_option(None, path), "Unknown - broken.jsonc");
}
