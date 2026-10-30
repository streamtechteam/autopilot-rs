use std::{
    fs,
    marker::PhantomData,
    path::{Component, Path, PathBuf},
};

use serde::{Serialize, de::DeserializeOwned};

use crate::error::{FsError, Result};
use crate::jsonc::parse;

pub trait RecordId {
    fn record_id(&self) -> &str;
}

#[derive(Debug)]
pub struct JsonFileStore<T> {
    dir: PathBuf,
    marker: PhantomData<fn() -> T>,
}

impl<T> Clone for JsonFileStore<T> {
    fn clone(&self) -> Self {
        Self {
            dir: self.dir.clone(),
            marker: PhantomData,
        }
    }
}

impl<T> JsonFileStore<T> {
    pub fn new(dir: impl Into<PathBuf>) -> Self {
        Self {
            dir: dir.into(),
            marker: PhantomData,
        }
    }

    pub fn dir(&self) -> &Path {
        &self.dir
    }

    pub fn ensure_dir(&self) -> Result<()> {
        fs::create_dir_all(&self.dir).map_err(|error| {
            FsError::Init(format!(
                "Failed to create storage directory {}: {error}",
                self.dir.display()
            ))
        })
    }

    pub fn file_paths(&self) -> Result<Vec<PathBuf>> {
        let entries = fs::read_dir(&self.dir).map_err(|error| {
            FsError::Init(format!(
                "Failed to read storage directory {}: {error}",
                self.dir.display()
            ))
        })?;
        let mut paths = Vec::new();

        for entry in entries {
            let path = entry
                .map_err(|error| {
                    FsError::Init(format!("Failed to read storage directory entry: {error}"))
                })?
                .path();
            if path.is_file() {
                paths.push(path);
            }
        }

        paths.sort();
        Ok(paths)
    }

    pub fn read_path(&self, path: impl AsRef<Path>) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let content = fs::read_to_string(path)?;
        parse_record(&content)
    }

    pub fn write_path(&self, path: impl AsRef<Path>, record: &T) -> Result<()>
    where
        T: Serialize,
    {
        let path = path.as_ref();
        self.ensure_member_path(path)?;
        let content = serde_json::to_string_pretty(record)?;
        fs::write(path, content)?;
        Ok(())
    }

    pub fn remove_path(&self, path: impl AsRef<Path>) -> Result<()> {
        let path = path.as_ref();
        self.ensure_member_path(path)?;
        fs::remove_file(path)?;
        Ok(())
    }

    fn ensure_member_path(&self, path: &Path) -> Result<()> {
        let relative = path
            .strip_prefix(self.dir.as_path())
            .map_err(|_| FsError::InvalidId(path.display().to_string()))?;

        if relative.as_os_str().is_empty()
            || relative.components().any(|component| {
                matches!(
                    component,
                    Component::ParentDir | Component::RootDir | Component::Prefix(_)
                )
            })
        {
            return Err(FsError::InvalidId(path.display().to_string()));
        }

        Ok(())
    }
}

impl<T> JsonFileStore<T>
where
    T: Serialize + DeserializeOwned + RecordId,
{
    pub fn list(&self) -> Result<Vec<(PathBuf, T)>> {
        let mut records = Vec::new();

        for path in self.file_paths()? {
            if let Ok(record) = self.read_path(&path) {
                records.push((path, record));
            }
        }

        records.sort_by(|(left_path, left), (right_path, right)| {
            left.record_id()
                .cmp(right.record_id())
                .then_with(|| left_path.cmp(right_path))
        });

        Ok(records)
    }

    pub fn get(&self, id: &str) -> Result<Option<T>> {
        Ok(self
            .list()?
            .into_iter()
            .find_map(|(_, record)| (record.record_id() == id).then_some(record)))
    }

    pub fn save(&self, record: &T) -> Result<PathBuf> {
        validate_id(record.record_id())?;
        self.ensure_dir()?;

        let mut matching = self.matching_paths(record.record_id())?;
        let path = if matching.is_empty() {
            self.unique_path(record.record_id())
        } else {
            matching.sort_by_key(|path| {
                let canonical =
                    path.file_stem().and_then(|stem| stem.to_str()) == Some(record.record_id());
                (!canonical, path.clone())
            });
            let path = matching.remove(0);
            for duplicate in matching {
                self.remove_path(&duplicate)?;
            }
            path
        };

        let content = serde_json::to_string_pretty(record)?;
        fs::write(&path, content)?;
        Ok(path)
    }

    pub fn delete(&self, id: &str) -> Result<bool> {
        validate_id(id)?;
        let mut deleted = false;

        for path in self.matching_paths(id)? {
            self.remove_path(&path)?;
            deleted = true;
        }

        Ok(deleted)
    }

    fn matching_paths(&self, id: &str) -> Result<Vec<PathBuf>> {
        Ok(self
            .list()?
            .into_iter()
            .filter_map(|(path, record)| (record.record_id() == id).then_some(path))
            .collect())
    }

    fn unique_path(&self, id: &str) -> PathBuf {
        let path = self.dir.join(format!("{id}.jsonc"));
        if !path.exists() {
            return path;
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        self.dir.join(format!("{id}-{timestamp}.jsonc"))
    }
}

fn parse_record<T: DeserializeOwned>(content: &str) -> Result<T> {
    serde_json::from_str(content)
        .or_else(|_| serde_json::from_str(&parse(content)))
        .map_err(FsError::Json)
}

fn validate_id(id: &str) -> Result<()> {
    if id.trim().is_empty()
        || id == "."
        || id == ".."
        || id.ends_with(' ')
        || id.ends_with('.')
        || id
            .chars()
            .any(|character| character.is_control() || r#"<>:"/\|?*"#.contains(character))
    {
        return Err(FsError::InvalidId(id.to_string()));
    }

    Ok(())
}

#[cfg(test)]
#[path = "storage.spec.rs"]
mod spec;
