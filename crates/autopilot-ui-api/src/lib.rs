use std::path::{Path, PathBuf};

use async_trait::async_trait;
use autopilot_core::{
    error::AutoPilotError, fs::get_jobs_dir, job::JobScheme, utilities::jsonc_parser::jsonc_parse,
};

#[async_trait]
pub trait JobApi: Send + Sync {
    async fn list_jobs(&self) -> Result<Vec<JobScheme>, AutoPilotError>;
    async fn get_job(&self, id: &str) -> Result<Option<JobScheme>, AutoPilotError>;
    async fn save_job(&self, job: JobScheme) -> Result<PathBuf, AutoPilotError>;
    async fn delete_job(&self, id: &str) -> Result<bool, AutoPilotError>;
}

#[derive(Clone, Debug)]
pub struct FileJobApi {
    jobs_dir: PathBuf,
}

impl FileJobApi {
    pub fn new() -> Result<Self, AutoPilotError> {
        let jobs_dir = get_jobs_dir()?;
        std::fs::create_dir_all(&jobs_dir).map_err(|error| {
            AutoPilotError::InvalidJob(format!(
                "Failed to create jobs directory {}: {error}",
                jobs_dir.display()
            ))
        })?;

        Ok(Self { jobs_dir })
    }

    pub fn with_dir(jobs_dir: impl Into<PathBuf>) -> Self {
        Self {
            jobs_dir: jobs_dir.into(),
        }
    }

    pub fn jobs_dir(&self) -> &Path {
        &self.jobs_dir
    }

    async fn job_files(&self) -> Result<Vec<(PathBuf, JobScheme)>, AutoPilotError> {
        let mut entries = tokio::fs::read_dir(&self.jobs_dir).await.map_err(|error| {
            AutoPilotError::InvalidJob(format!(
                "Failed to read jobs directory {}: {error}",
                self.jobs_dir.display()
            ))
        })?;
        let mut jobs = Vec::new();

        while let Some(entry) = entries.next_entry().await.map_err(|error| {
            AutoPilotError::InvalidJob(format!(
                "Failed to read jobs directory {}: {error}",
                self.jobs_dir.display()
            ))
        })? {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            let Ok(content) = tokio::fs::read_to_string(&path).await else {
                continue;
            };
            let Ok(job) = parse_job(&content) else {
                continue;
            };
            jobs.push((path, job));
        }

        jobs.sort_by(|(left_path, left), (right_path, right)| {
            left.id
                .cmp(&right.id)
                .then_with(|| left_path.cmp(right_path))
        });

        Ok(jobs)
    }

    async fn matching_jobs(&self, id: &str) -> Result<Vec<PathBuf>, AutoPilotError> {
        Ok(self
            .job_files()
            .await?
            .into_iter()
            .filter_map(|(path, job)| (job.id == id).then_some(path))
            .collect())
    }

    async fn unique_path(&self, id: &str) -> PathBuf {
        let path = self.jobs_dir.join(format!("{id}.jsonc"));
        if !path.exists() {
            return path;
        }

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|duration| duration.as_nanos())
            .unwrap_or_default();
        self.jobs_dir.join(format!("{id}-{timestamp}.jsonc"))
    }
}

#[async_trait]
impl JobApi for FileJobApi {
    async fn list_jobs(&self) -> Result<Vec<JobScheme>, AutoPilotError> {
        Ok(self
            .job_files()
            .await?
            .into_iter()
            .map(|(_, job)| job)
            .collect())
    }

    async fn get_job(&self, id: &str) -> Result<Option<JobScheme>, AutoPilotError> {
        Ok(self
            .job_files()
            .await?
            .into_iter()
            .find_map(|(_, job)| (job.id == id).then_some(job)))
    }

    async fn save_job(&self, job: JobScheme) -> Result<PathBuf, AutoPilotError> {
        validate_job(&job)?;
        tokio::fs::create_dir_all(&self.jobs_dir)
            .await
            .map_err(|error| {
                AutoPilotError::InvalidJob(format!(
                    "Failed to create jobs directory {}: {error}",
                    self.jobs_dir.display()
                ))
            })?;

        let mut matching = self.matching_jobs(&job.id).await?;
        let path = if matching.is_empty() {
            self.unique_path(&job.id).await
        } else {
            matching.sort_by_key(|path| {
                let canonical =
                    path.file_stem().and_then(|stem| stem.to_str()) == Some(job.id.as_str());
                (!canonical, path.clone())
            });
            let path = matching.remove(0);
            for duplicate in matching {
                tokio::fs::remove_file(&duplicate).await.map_err(|error| {
                    AutoPilotError::InvalidJob(format!(
                        "Failed to remove duplicate job file {}: {error}",
                        duplicate.display()
                    ))
                })?;
            }
            path
        };

        let content = serde_json::to_string_pretty(&job)
            .map_err(|error| AutoPilotError::Json(error.to_string()))?;
        tokio::fs::write(&path, content).await.map_err(|error| {
            AutoPilotError::InvalidJob(format!(
                "Failed to write job file {}: {error}",
                path.display()
            ))
        })?;

        Ok(path)
    }

    async fn delete_job(&self, id: &str) -> Result<bool, AutoPilotError> {
        validate_id(id)?;
        let mut deleted = false;

        for path in self.matching_jobs(id).await? {
            tokio::fs::remove_file(&path).await.map_err(|error| {
                AutoPilotError::InvalidJob(format!(
                    "Failed to remove job file {}: {error}",
                    path.display()
                ))
            })?;
            deleted = true;
        }

        Ok(deleted)
    }
}

fn parse_job(content: &str) -> Result<JobScheme, serde_json::Error> {
    serde_json::from_str(content).or_else(|_| serde_json::from_str(&jsonc_parse(content)))
}

fn validate_job(job: &JobScheme) -> Result<(), AutoPilotError> {
    validate_id(&job.id)
}

fn validate_id(id: &str) -> Result<(), AutoPilotError> {
    if id.trim().is_empty()
        || id == "."
        || id == ".."
        || id.ends_with(' ')
        || id.ends_with('.')
        || id
            .chars()
            .any(|character| character.is_control() || r#"<>:"/\|?*"#.contains(character))
    {
        return Err(AutoPilotError::InvalidJob(format!("Invalid job id '{id}'")));
    }

    Ok(())
}

#[cfg(test)]
#[path = "ui_api.spec.rs"]
mod spec;
