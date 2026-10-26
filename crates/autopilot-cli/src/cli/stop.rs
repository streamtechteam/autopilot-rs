use autopilot_core::checks::address;
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
    let address = address();
    let mut stream = TcpStream::connect(&address)?;
    let _ = stream.set_read_timeout(Some(std::time::Duration::from_secs(2)));
    let _ = stream.set_write_timeout(Some(std::time::Duration::from_secs(2)));

    let request = format!(
        "POST /shutdown HTTP/1.1\r\nHost: {address}\r\nContent-Length: 0\r\nConnection: close\r\n\r\n"
    );

    if stream.write_all(request.as_bytes()).is_err() {
        eprintln!("Warning: failed to stop autopilot");
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
