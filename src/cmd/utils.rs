extern crate serde;
extern crate serde_json;
extern crate time;

use std::error::Error;
use std::io::Write;
use std::net::UdpSocket;
use std::process::exit;
use std::time::Duration;
use types::{CurrentTime, Event, EventsPayload};

pub fn update_telemetry<T>(value: &T) -> Result<(), std::io::Error>
where
    T: serde::Serialize,
{
    let serialized = serde_json::to_string(value).unwrap();
    let payload = serialized.as_bytes();
    let socket = try!(UdpSocket::bind("127.0.0.1:0"));
    try!(socket.send_to(&payload, "127.0.0.1:48656"));
    Ok(())
}

pub fn abort(exit_code: i32, message: String) -> ! {
    update_telemetry(&EventsPayload {
        events: vec![Event {
            event_name: "hull_fatal_error".to_string(),
            event_data: CurrentTime {
                created_at: time::get_time().sec,
            },
        }],
    })
    .ok();
    match stderr_write(message) {
        Ok(_) => exit(exit_code),
        Err(e) => unexpected_io_error(e),
    };
}

fn stderr_write(message: String) -> std::io::Result<()> {
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();

    try!(handle.write(message.as_bytes()));
    try!(handle.write(b"\n"));
    Ok(())
}

fn unexpected_io_error(err: std::io::Error) -> ! {
    println!("failure: {}", err.description().to_string());
    match err.raw_os_error() {
        Some(code) => exit(code),
        None => exit(1),
    }
}

pub fn duration_in_millis(duration: Duration) -> f64 {
    return ((duration.as_secs() as f64) * 1_000.0)
        + ((duration.subsec_nanos() as f64) / 1_000_000.0);
}
