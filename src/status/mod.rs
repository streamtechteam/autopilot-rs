use serde::{Deserialize, Serialize};

pub mod get;
pub mod set;

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct StatusLog {
    pub time: String,
    pub statuses: Vec<JobStatus>,
}

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JobStatus {
    pub id: String,
    pub name: String,
    pub status: Status,
}

impl JobStatus {
    pub fn new(id: String, name: String, status: Status) -> Self {
        JobStatus { id, name, status }
    }
}
#[derive(Debug, Deserialize, Serialize, Clone)]
pub enum Status {
    Pending,
    Running,
    Completed,
    Success,
    Failed,
    Unknown,
}
