extern crate crossbeam_channel as channel;
extern crate nix;
extern crate signal_hook;

use self::signal_hook::{SIGABRT, SIGINT, SIGHUP, SIGQUIT, SIGTERM, SIGUSR1, SIGUSR2, SIGPIPE};
use std::thread;

pub struct SignalHandler {
    tx: channel::Sender<i32>,
    rx: channel::Receiver<i32>,
}

impl SignalHandler {
    pub fn new() -> SignalHandler {
        let (tx, rx) = channel::unbounded();
        SignalHandler { tx, rx }
    }

    pub fn receiver(&mut self) -> channel::Receiver<i32> {
        self.rx.clone()
    }

    pub fn handle_in_thread(&mut self) -> Result<(), std::io::Error> {
        let tx = self.tx.clone();
        // Trapping but not forwarding SIGINT.
        // In interactive mode, SIGINT is sent to the process group after the
        // action has been initiated by the user from the keyboard.
        // Should the need arise for daemon mode, we should make some effort
        // in detecting of daemon mode is used, trap the signal and forward.
        let signals = signal_hook::iterator::Signals::new(&[
            SIGINT, SIGTERM, SIGPIPE, SIGQUIT, SIGHUP, SIGABRT, SIGUSR1, SIGUSR2,
        ])?;
        thread::spawn(move || {
            for sig in signals.forever() {
                if sig == SIGINT {
                    debug!("Trapped SIGINT. Ignoring.");
                    continue;
                }
                if tx.send(sig).is_err() {
                    //TODO: handle error.
                    break;
                }
            }
        });
        Ok(())
    }
}

pub fn kill(process_id: u32, signal: i32) -> Result<(), std::io::Error> {
    let nix_pid = nix::unistd::Pid::from_raw(process_id as i32);
    let nix_sig = try!(nix::sys::signal::Signal::from_c_int(signal).map_err(nix_err_to_io_err));
    try!(nix::sys::signal::kill(nix_pid, Some(nix_sig)).map_err(nix_err_to_io_err));

    Ok(())
}

fn nix_err_to_io_err(err: nix::Error) -> std::io::Error {
    match err {
        nix::Error::Sys(err_no) => std::io::Error::from(err_no),
        _ => std::io::Error::new(std::io::ErrorKind::InvalidData, err),
    }
}
