use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StatusLog {
    pub time: String,
    pub statuses: Vec<JobStatusStruct>,
}

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

struct Inner {
    log: StatusLog,
    index: HashMap<String, usize>,
}

#[derive(Clone)]
pub struct StateManager {
    inner: Arc<RwLock<Inner>>,
}

impl StateManager {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(Inner {
                log: StatusLog {
                    time: chrono::Local::now().to_string(),
                    statuses: Vec::new(),
                },
                index: HashMap::new(),
            })),
        }
    }

    pub async fn set(&self, id: String, name: Option<String>, status: JobStatusEnum) {
        let mut inner = self.inner.write().await;
        if let Some(&position) = inner.index.get(&id) {
            inner.log.statuses[position].status = status;
        } else {
            let position = inner.log.statuses.len();
            inner.index.insert(id.clone(), position);
            inner.log.statuses.push(JobStatusStruct {
                id,
                name: name.unwrap_or_default(),
                status,
            });
        }
    }

    pub async fn get_all(&self) -> Vec<JobStatusStruct> {
        self.inner.read().await.log.statuses.clone()
    }

    pub async fn get_status_log(&self) -> StatusLog {
        self.inner.read().await.log.clone()
    }

    pub async fn get_state_by_id(&self, id: String) -> Option<JobStatusEnum> {
        let inner = self.inner.read().await;
        let position = *inner.index.get(&id)?;
        inner
            .log
            .statuses
            .get(position)
            .map(|state| state.status.clone())
    }

    pub async fn get_state_struct_by_id(&self, id: String) -> Option<JobStatusStruct> {
        let inner = self.inner.read().await;
        let position = *inner.index.get(&id)?;
        inner.log.statuses.get(position).cloned()
    }
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JobStatusStruct {
    pub id: String,
    pub name: String,
    pub status: JobStatusEnum,
}

impl JobStatusStruct {
    pub fn new(id: String, name: String, status: JobStatusEnum) -> Self {
        JobStatusStruct { id, name, status }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub enum JobStatusEnum {
    /// Job is queued but not yet started
    Pending,
    /// Job is actively executing
    Running,
    /// Job completed successfully
    Completed,
    /// Job failed during execution
    Failed,
    /// Job was intentionally stopped
    Cancelled,
    /// Job is waiting for dependencies or conditions
    Waiting,
    /// Job didnt run due to conditions not being met
    Unsatisfied,
    /// Status cannot be determined (default state)
    Unknown,
    /// Job is scheduled but not yet started
    Scheduled,
}

#[cfg(test)]
#[path = "status.spec.rs"]
mod spec;
