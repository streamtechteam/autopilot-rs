use autopilot_core::checks::check_if_running;

pub fn status() {
    match check_if_running() {
        true => {
            println!("Autopilot is running");
        }
        false => {
            println!("Autopilot is not running");
        }
    }
}
