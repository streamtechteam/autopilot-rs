use log::error;


pub fn sync_run(command: String) {
    if let Err(e) = duct_sh::sh_dangerous(&command).run() {
        error!("Failed to run task '{}': {}", command, e);
    }
}

pub async fn async_run(command: String) {
    sync_run(command);
}
