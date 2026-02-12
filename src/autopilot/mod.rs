use tokio_cron_scheduler::JobScheduler;

use crate::{cron::init::init_time_check, logging::init_logging};

pub struct AutoPilot {
    pub scheduler: JobScheduler,
}

impl AutoPilot {
    pub async fn new() -> Self {
        Self {
            scheduler: init_time_check().await.expect("failed to init cron"),
        }
    }
    pub fn start(&mut self, verbose: bool) {
        Self::prepare_logging(verbose);
    }
    pub fn prepare_logging(verbose: bool) {
        init_logging(verbose);
    }
}
