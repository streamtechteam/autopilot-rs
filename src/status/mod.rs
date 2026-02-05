use serde::{Deserialize, Serialize};

pub mod get;
pub mod set;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StatusLog {
    pub time: String,
    pub statuses: Vec<JobStatusStruct>,
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
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum JobStatusEnum {
    /// Job is queued but not yet started
    Pending,
    /// Job is actively executing
    Running,
    /// Job completed successfully
    Success,
    /// Job failed during execution
    Failed,
    /// Job was intentionally stopped
    Cancelled,
    /// Job is waiting for dependencies or conditions
    Waiting,
    // Job didnt run due to conditions not being met
    Unsatisfied,
    /// Status cannot be determined (default state)
    Unknown,
}
