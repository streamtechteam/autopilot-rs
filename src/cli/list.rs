// use log::info;
use colored::*;

use crate::{
    job::get::get_jobs,
    state::get::{get_job_status, get_status_log},
};

pub fn list() {
    let jobs = get_jobs();

    for job in jobs {
        println!(
            "* id {} - {} - {:?}",
            job.id,
            job.name,
            get_job_status(job.id.clone())
        );
    }
    // println!("{:?}", get_state_items())
}
