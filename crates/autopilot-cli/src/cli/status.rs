use autopilot_core::checks::check_if_running;

pub fn status() {
    println!("{}", status_message(check_if_running()));
}

fn status_message(running: bool) -> &'static str {
    if running {
        "Autopilot is running"
    } else {
        "Autopilot is not running"
    }
}

#[cfg(test)]
#[path = "status.spec.rs"]
mod spec;
