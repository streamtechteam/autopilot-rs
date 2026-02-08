use duct::cmd;

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

pub fn check_if_running() -> bool {
    let pid_str = duct_sh::sh_dangerous("pgrep autopilot")
        .read()
        .expect("if you are seeing this, something is clearly wrong with your device!");
    let pids: Vec<u32> = match pid_str.clone().parse::<u32>() {
        Ok(pid) => {
            vec![pid]
        }
        Err(_) => pid_str
            .split_whitespace()
            .map(|value| value.parse().expect("You shouldnt have seen this"))
            .collect(),
    };
    if pids.len() > 1 { true } else { false }
}
