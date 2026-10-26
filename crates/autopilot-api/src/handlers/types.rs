use serde::Serialize;

use autopilot_core::status::JobStatusStruct;

#[cfg(test)]
use autopilot_core::status::StateManager;

#[derive(Serialize, Debug)]
pub struct JobResponse {
    pub(crate) id: String,
    pub(crate) name: String,
    pub(crate) description: String,
    pub(crate) loaded: bool,
    pub(crate) status: String,
}

impl JobResponse {
    #[cfg(test)]
    async fn from_job_and_sm(job: &autopilot_core::job::Job, sm: StateManager) -> Self {
        JobResponse {
            id: job.id.clone(),
            loaded: job.loaded,
            name: job.name.clone(),
            description: job.description.clone(),
            status: format!("{:?}", sm.get_state_by_id(job.id.clone()).await),
        }
    }
}

impl From<&JobStatusStruct> for JobResponse {
    fn from(job: &JobStatusStruct) -> Self {
        JobResponse {
            id: job.id.clone(),
            name: job.name.clone(),
            loaded: true,
            description: job.name.clone(),
            status: format!("{:?}", job.status),
        }
    }
}

#[cfg(test)]
#[path = "types.spec.rs"]
mod spec;
