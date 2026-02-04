// use log::info;

use crate::status::get::get_status_log;

pub fn list() {
    let status_log = get_status_log();

    for log in status_log.statuses {
        println!("* id {} - {} - {:?}", log.id, log.name, log.status);
    }
    // println!("{:?}", get_state_items())
}
