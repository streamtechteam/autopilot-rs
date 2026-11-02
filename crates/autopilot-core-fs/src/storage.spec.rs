use serde::{Deserialize, Serialize};

use super::*;

#[derive(Debug, Deserialize, PartialEq, Serialize)]
struct Record {
    id: String,
    name: String,
}

impl RecordId for Record {
    fn record_id(&self) -> &str {
        &self.id
    }
}

struct TestDir {
    path: PathBuf,
}

impl TestDir {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "autopilot-core-fs-store-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        Self { path }
    }

    fn store(&self) -> JsonFileStore<Record> {
        JsonFileStore::new(&self.path)
    }
}

impl Drop for TestDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.path);
    }
}

fn record(id: &str, name: &str) -> Record {
    Record {
        id: id.to_string(),
        name: name.to_string(),
    }
}

#[test]
fn saves_lists_and_gets_records() {
    let dir = TestDir::new();
    let store = dir.store();

    let path = store.save(&record("record_1", "First")).unwrap();

    assert!(path.exists());
    assert_eq!(store.list().unwrap().len(), 1);
    assert_eq!(store.get("record_1").unwrap().unwrap().name, "First");
    assert!(store.get("missing").unwrap().is_none());
}

#[test]
fn save_upserts_by_id_and_removes_duplicates() {
    let dir = TestDir::new();
    let store = dir.store();
    let first = store.save(&record("record_1", "First")).unwrap();
    let duplicate = dir.path.join("duplicate.jsonc");
    fs::write(
        &duplicate,
        serde_json::to_string(&record("record_1", "Old")).unwrap(),
    )
    .unwrap();

    let second = store.save(&record("record_1", "Updated")).unwrap();

    assert_eq!(first, second);
    assert!(!duplicate.exists());
    assert_eq!(store.list().unwrap().len(), 1);
    assert_eq!(store.get("record_1").unwrap().unwrap().name, "Updated");
}

#[test]
fn reports_invalid_files_with_their_path() {
    let dir = TestDir::new();
    let store = dir.store();
    store.ensure_dir().unwrap();
    fs::write(
        dir.path.join("commented.jsonc"),
        r#"{
            // comment
            "id": "commented",
            "name": "Commented",
        }"#,
    )
    .unwrap();
    fs::write(dir.path.join("broken.jsonc"), "{ not json").unwrap();

    let error = store.list().unwrap_err();
    assert!(
        error.to_string().contains("broken.jsonc"),
        "unexpected error: {error}"
    );
    assert_eq!(
        store
            .read_path(dir.path.join("commented.jsonc"))
            .unwrap()
            .id,
        "commented"
    );
}

#[test]
fn failed_write_leaves_the_previous_entry_and_no_temp_file() {
    let dir = TestDir::new();
    let store = dir.store();
    store.ensure_dir().unwrap();
    let blocked = dir.path.join("blocked");
    fs::create_dir(&blocked).unwrap();

    assert!(
        store
            .write_path(&blocked, &record("blocked", "Blocked"))
            .is_err()
    );
    assert!(blocked.is_dir());

    let leftovers: Vec<PathBuf> = fs::read_dir(&dir.path)
        .unwrap()
        .filter_map(|entry| entry.ok().map(|entry| entry.path()))
        .filter(|path| path.extension().and_then(|ext| ext.to_str()) == Some("tmp"))
        .collect();
    assert!(
        leftovers.is_empty(),
        "temp files left behind: {leftovers:?}"
    );
}

#[test]
fn writes_do_not_leave_temporary_files() {
    let dir = TestDir::new();
    let store = dir.store();
    let path = store.save(&record("record_1", "First")).unwrap();

    assert_eq!(
        fs::read_to_string(&path).unwrap(),
        serde_json::to_string_pretty(&record("record_1", "First")).unwrap()
    );
    assert!(fs::read_dir(&dir.path).unwrap().all(|entry| {
        entry
            .unwrap()
            .path()
            .extension()
            .and_then(|ext| ext.to_str())
            != Some("tmp")
    }));
}

#[test]
fn delete_reports_whether_a_record_existed() {
    let dir = TestDir::new();
    let store = dir.store();
    store.save(&record("record_1", "First")).unwrap();

    assert!(store.delete("record_1").unwrap());
    assert!(!store.delete("record_1").unwrap());
    assert!(store.list().unwrap().is_empty());
}

#[test]
fn rejects_paths_outside_the_store_directory() {
    let dir = TestDir::new();
    let store = dir.store();
    let outside = dir.path.join("..").join("outside.jsonc");

    assert!(
        store
            .write_path(&outside, &record("outside", "Outside"))
            .is_err()
    );
    assert!(store.remove_path(&outside).is_err());

    let relative_store = JsonFileStore::<Record>::new("relative-jobs");
    let relative_outside = Path::new("relative-jobs/../outside.jsonc");
    assert!(
        relative_store
            .write_path(relative_outside, &record("outside", "Outside"))
            .is_err()
    );
    assert!(relative_store.remove_path(relative_outside).is_err());
}

#[test]
fn invalid_ids_are_rejected() {
    let dir = TestDir::new();
    let store = dir.store();

    for id in [
        "",
        " ",
        ".",
        "..",
        "nested/record",
        "nested\\record",
        "bad:name",
        "bad*name",
        "trailing ",
        "trailing.",
    ] {
        assert!(store.save(&record(id, "Invalid")).is_err(), "{id}");
        assert!(store.delete(id).is_err(), "{id}");
    }
}
