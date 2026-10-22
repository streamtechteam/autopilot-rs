use autopilot_core::checks::port;
use colored::Colorize;
use core::result::Result::Ok;
use log::info;
use std::io::{Read, Write};
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

    // Wait for the daemon's response before dropping the socket. Dropping
    // immediately sends a RST that can discard the shutdown request before
    // the server processes it.
    let mut response = String::new();
    let _ = stream.read_to_string(&mut response);

    Ok(())
}

#[cfg(test)]
#[path = "stop.spec.rs"]
mod spec;
