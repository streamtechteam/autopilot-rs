use super::*;

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "autopilot-core-fs-paths-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        Self { path }
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

#[test]
fn builds_paths_relative_to_the_root() {
    let dir = TestDir::new();
    let paths = Paths::from_root(&dir.path);

    assert_eq!(paths.root(), dir.path);
    assert_eq!(paths.config_file(), dir.path.join("autopilot.jsonc"));
    assert_eq!(paths.jobs_dir(), dir.path.join("jobs"));
    assert_eq!(paths.logs_dir(), dir.path.join("logs"));
}

#[test]
fn ensure_creates_directories_and_config_once() {
    let dir = TestDir::new();
    let paths = Paths::from_root(&dir.path);

    paths.ensure().unwrap();
    paths.ensure().unwrap();

    assert!(paths.root().is_dir());
    assert!(paths.jobs_dir().is_dir());
    assert!(paths.logs_dir().is_dir());
    assert_eq!(fs::read_to_string(paths.config_file()).unwrap(), "{}");
}
