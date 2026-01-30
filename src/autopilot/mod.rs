use tokio_cron_scheduler::JobScheduler;

use crate::{cron::init::init_time_check, logging::init_logging};

pub struct AutoPilot {
    pub scheduler: JobScheduler,
}

impl AutoPilot {
    pub async fn new() -> Self {
        Self {
            scheduler: init_time_check().await,
        }
        // self.scheduler =
    }
    pub fn start(&mut self) {
        Self::prepare_logging();
    }
    pub fn prepare_logging() {
        init_logging();
    }
}
