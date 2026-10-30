use std::path::{Path, PathBuf};

use async_trait::async_trait;
use autopilot_core_fs::FsError;
use autopilot_core_fs::config;
use autopilot_core_fs::storage::{JsonFileStore, RecordId};

use autopilot_core_error::AutoPilotError;

use crate::job::JobScheme;

#[async_trait]
#[allow(clippy::double_must_use)]
pub trait JobRepository: Send + Sync {
    async fn list_jobs(&self) -> Result<Vec<JobScheme>, AutoPilotError>;
    async fn get_job(&self, id: &str) -> Result<Option<JobScheme>, AutoPilotError>;
    async fn save_job(&self, job: JobScheme) -> Result<PathBuf, AutoPilotError>;
    async fn delete_job(&self, id: &str) -> Result<bool, AutoPilotError>;
}

#[derive(Clone, Debug)]
pub struct FsJobRepository {
    store: JsonFileStore<JobScheme>,
}

impl FsJobRepository {
    pub fn new() -> Result<Self, AutoPilotError> {
        let repository = Self::with_dir(config::current()?.jobs_dir());
        repository.store.ensure_dir()?;
        Ok(repository)
    }

    pub fn with_dir(jobs_dir: impl Into<PathBuf>) -> Self {
        Self {
            store: JsonFileStore::new(jobs_dir),
        }
    }

    pub fn jobs_dir(&self) -> &Path {
        self.store.dir()
    }
}

impl RecordId for JobScheme {
    fn record_id(&self) -> &str {
        &self.id
    }
}

#[async_trait]
impl JobRepository for FsJobRepository {
    async fn list_jobs(&self) -> Result<Vec<JobScheme>, AutoPilotError> {
        let store = self.store.clone();
        let records = run_blocking(move || store.list()).await?;
        Ok(records.into_iter().map(|(_, job)| job).collect())
    }

    async fn get_job(&self, id: &str) -> Result<Option<JobScheme>, AutoPilotError> {
        let store = self.store.clone();
        let id = id.to_string();
        run_blocking(move || store.get(&id)).await
    }

    async fn save_job(&self, job: JobScheme) -> Result<PathBuf, AutoPilotError> {
        let store = self.store.clone();
        run_blocking(move || store.save(&job)).await
    }

    async fn delete_job(&self, id: &str) -> Result<bool, AutoPilotError> {
        let store = self.store.clone();
        let id = id.to_string();
        run_blocking(move || store.delete(&id)).await
    }
}

async fn run_blocking<T, F>(task: F) -> Result<T, AutoPilotError>
where
    T: Send + 'static,
    F: FnOnce() -> std::result::Result<T, FsError> + Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|error| AutoPilotError::JobExecution(format!("Storage task failed: {error}")))?
        .map_err(AutoPilotError::from)
}
