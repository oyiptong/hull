use std::env;
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::time::Instant;
use std::io::{self, Write};
use std::process::{exit, Command};
#[macro_use] extern crate log;
extern crate env_logger;


fn stderr_write(message :String) -> io::Result<()> {
    let stderr = io::stderr();
    let mut handle = stderr.lock();

    try!(handle.write(message.as_bytes()));
    try!(handle.write(b"\n"));
    Ok(())
}

/// Returns a shell PATH environment string minus a given directory
fn remove_dir_from_path(dir :String, path_str :String) -> String{
    let mut child_paths = Vec::new(); 
    for path in path_str.split(":") {
        if path != dir {
            child_paths.push(path);
        }
    }

    return child_paths.join(":");
}


/// Returns whether a command and hull's binary paths are the same
/// # Panics
/// Will panic if the command path is relative and does not resolve
fn paths_equivalent(binary_path :PathBuf, cur_cmd :String, cur_path :&Path) -> bool {
    let command_path;

    if cur_cmd.contains("/") {
        let mut cur_path_buf = cur_path.to_path_buf();
        cur_path_buf.push(cur_cmd);
        command_path = cur_path_buf.canonicalize().unwrap();
    } else {
        command_path = PathBuf::from(cur_cmd);
    }

    debug!("comparing command_path:'{}' binary_path:'{}'",
            command_path.display(),
            binary_path.display()
    );

    let binary_path = binary_path.canonicalize().unwrap();

    return command_path == binary_path;
}

fn duration_in_millis(duration :Duration) -> f64 {
    return ((duration.as_secs() as f64) * 1_000.0) +
        ((duration.subsec_nanos() as f64) / 1_000_000.0);
}

fn main() {
    let now = Instant::now();
    env_logger::init().unwrap();
    let args: Vec<String> = env::args().collect();

    let ref cmd = args[0];
    let cmd_args = &args[1 .. args.len()];

    // remove binary path from PATH so the shell can resolve the next command location
    let binary_path = env::current_exe().unwrap();

    let binary_dir = String::from(
        binary_path.parent().unwrap()
            .to_string_lossy()
            .into_owned()
    );

    // current shell invocation path
    let cur_dir = env::current_dir().unwrap();
    let cur_path = cur_dir.as_path();

    if paths_equivalent(binary_path, cmd.clone(), cur_path) {
        error!("Error: Unwilling to run program recursively. Please check your paths.");
        exit(1);
    }

    debug!("command: '{}' args: '{:#?}'", cmd, cmd_args);

    let path_str = env::var("PATH").unwrap();
    let new_path_str = remove_dir_from_path(binary_dir, path_str);

    let boot_duration = now.elapsed();

    let result = Command::new(cmd)
        .args(cmd_args)
        .env("PATH", new_path_str)
        .current_dir(cur_path)
        .status();

    if result.is_ok() {
        let status = result.unwrap();

        if !status.success() {
            let code = status.code();
            match code {
                Some(status_code) => exit(status_code),
                None => exit(1),
            }
        }
    } else {
        let err = result.unwrap_err();
        let error_code = err.raw_os_error().unwrap();
        stderr_write(format!("{} : {}", cmd, err.to_string())).unwrap();
        exit(error_code);
    }

    let total_duration = now.elapsed();

    let boot_time = duration_in_millis(boot_duration);
    let total_time = duration_in_millis(total_duration);
    info!("boot_time: {} ms\ntotal_time: {} ms", boot_time, total_time);
}
