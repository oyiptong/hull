extern crate crossbeam_channel as channel;

use std::path::PathBuf;
use std::process::Command;
use std::thread;

#[derive(Clone)]
pub struct CommandParams {
    pub cmd: String,
    pub args: Vec<String>,
    pub path: String,
    pub dir: PathBuf,
}

pub struct Process {
    params: CommandParams,
    tx: channel::Sender<i32>,
    rx: channel::Receiver<i32>,
}

impl Process {
    pub fn new(params: CommandParams) -> Process {
        let (tx, rx) = channel::bounded(1);
        Process { tx, rx, params }
    }

    pub fn receiver(&mut self) -> channel::Receiver<i32> {
        self.rx.clone()
    }

    pub fn run_in_thread(&mut self) -> Result<u32, std::io::Error> {
        let params = self.params.clone();
        let tx = self.tx.clone();

        let mut child = Command::new(params.cmd)
            .args(params.args)
            .env("PATH", params.path)
            .current_dir(params.dir)
            .spawn()?;

        let pid = child.id();
        thread::spawn(move || {
            let status_code = match child.wait() {
                Ok(status) => match status.code() {
                    Some(status_code) => status_code,
                    // None as status code optional value means the child was terminated by signal.
                    None => -1,
                },
                Err(e) => match e.raw_os_error() {
                    Some(error_code) => error_code,
                    None => -1,
                },
            };
            if tx.send(status_code).is_err() {
                error!("Process watcher failed to send status");
                return;
            }
        });
        Ok(pid)
    }
}
