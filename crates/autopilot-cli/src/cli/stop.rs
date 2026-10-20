use autopilot_core::checks::port;
use colored::Colorize;
use core::result::Result::Ok;
use log::info;
use std::io::Write;
use std::net::TcpStream;

use autopilot_core::language;

pub fn stop(quiet: bool) -> Result<(), std::io::Error> {
    if !quiet {
        info!("{}", language::en_us::AUTOPILOT_SHUTDOWN.yellow());
    }
    let mut stream = match TcpStream::connect(format!("localhost:{}", port())) {
        Ok(stream) => stream,
        Err(err) => return Err(err),
    };

    let request = format!(
        "GET /shutdown HTTP/1.1\r\nHost: localhost:{}\r\nConnection: close\r\n\r\n",
        port()
    );

    if stream.write_all(request.as_bytes()).is_err() {
        eprintln!("Warning: failed to stop auto_pilot");
    }

    Ok(())
}
