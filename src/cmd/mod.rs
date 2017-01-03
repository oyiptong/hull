extern crate time;
extern crate serde;
extern crate serde_json;

use std;
use std::time::Duration;
use std::path::{Path, PathBuf};
use std::io::{self, Write};
use std::process::exit;
use std::net::UdpSocket;

include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

pub fn stderr_write(message :String) -> io::Result<()> {
    let stderr = std::io::stderr();
    let mut handle = stderr.lock();

    try!(handle.write(message.as_bytes()));
    try!(handle.write(b"\n"));
    Ok(())
}


pub fn update_telemetry<T>(value: &T) -> Result<(), io::Error>
    where T: serde::Serialize,
{
    let serialized = serde_json::to_string(value).unwrap();
    let payload = serialized.as_bytes();
    let socket = try!(UdpSocket::bind("127.0.0.1:0"));
    try!(socket.send_to(&payload, "127.0.0.1:48656"));
    Ok(())
}


pub fn abort(exit_code :i32, message :String) -> ! {
    update_telemetry(&Event {
        event_name: "hull_fatal_error".to_string(),
        event_data: CurrentTime {
            created_at: time::get_time().sec
        }
    }).ok();
    match stderr_write(message) {
        Ok(_) => exit(exit_code),
        Err(e) => unexpected_io_error(e),
    };
}


pub fn unexpected_io_error(err :std::io::Error) -> ! {
    println!("failure: {}", err.to_string());
    exit(1);
}


/// Returns a shell PATH environment string minus a given directory
pub fn remove_dir_from_path(dir :String, path_str :String) -> String{
    let mut child_paths = Vec::new(); 
    for path in path_str.split(":") {
        if path != dir {
            child_paths.push(path);
        }
    }

    return child_paths.join(":");
}


/// Returns whether a command and hull's binary paths are the same
///
/// `binary_path` cannot have `.` or `..`. it doesn't need to be strictly absolute,
/// as it will follow the rules for `std::fs::canonicalize`, which will pre-pend the current
/// working directory if need be.
///
/// # Panics
///
/// Will panic if either `binary_path` or the command path is relative and do not resolve
pub fn paths_equivalent(binary_path :PathBuf, cur_cmd :String, cur_path :&Path) -> Result<bool, io::Error> {
    let command_path;

    if cur_cmd.contains("/") {
        let mut cur_path_buf = cur_path.to_path_buf();
        cur_path_buf.push(cur_cmd);
        command_path = try!(cur_path_buf.canonicalize());
    } else {
        command_path = PathBuf::from(cur_cmd);
    }

    debug!("comparing command_path:'{}' binary_path:'{}'",
            command_path.display(),
            binary_path.display()
    );

    // still need to canonicalize even if absolute
    // e.g. /tmp/ when canonicalize may become /private/tmp/ on some OS's
    let binary_path = binary_path.canonicalize().unwrap();

    return Ok(command_path == binary_path);
}

pub fn duration_in_millis(duration :Duration) -> f64 {
    return ((duration.as_secs() as f64) * 1_000.0) +
        ((duration.subsec_nanos() as f64) / 1_000_000.0);
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs::{self, File};
    use std::path::PathBuf;
    use super::*;
    extern crate tempdir;
    use self::tempdir::TempDir;

    macro_rules! t {
        ($e:expr) => (match $e { Ok(n) => n, Err(e) => panic!("error: {}", e) })
    }

    fn in_tmpdir<F>(f: F) where F: FnOnce(&TempDir) {
        let tmpdir = t!(TempDir::new("test"));
        assert!(env::set_current_dir(tmpdir.path()).is_ok());

        f(&tmpdir);
        t!(tmpdir.close());
    }

    fn ensure_files_created(root :PathBuf, paths :&[PathBuf]) {
        for path in paths.into_iter() {
            let filename = &root.join(path);
            if path.to_string_lossy().contains("/") {
                let parent_dir = filename.parent().unwrap();
                // could check if directory exists before creating, but `.exists()` does not resolve
                // relative paths
                match fs::create_dir_all(parent_dir) {
                    Ok(_) => {},
                    Err(_) => {
                        // might get an error in case directory exists. ignore
                    },
                };
            }

            if !filename.exists() {
                debug!("creating: {}", filename.display());
                t!(File::create(filename));
            }
        }
    }

    #[test]
    fn remove_dir_from_path_valid_paths() {
        let inputs = [
            ["/home/user/.bin", "/home/user/.bin:/bin", "/bin"],
            ["/home/user/.bin", "/bin", "/bin"],
            ["/home/user/.bin", "/home/user/.bin:/bin:/home/user/.bin", "/bin"],
            ["/home/user/.bin", "", ""],
            ["", "/bin", "/bin"],
        ];

        for input in inputs.into_iter() {
            let dir = String::from(input[0]);
            let path_str = String::from(input[1]);
            let expected = input[2];

            assert_eq!(remove_dir_from_path(dir, path_str), expected);
        }
    }

    #[test]
    fn paths_equivalent_inputs() {
        let inputs = [
            ["prog", "./prog", "", "t"],
            ["foo/prog", "./foo/prog", "", "t"],
            ["foo/bar/prog", "./prog", "foo/bar", "t"],
            ["foo/prog", "../prog", "foo/bar", "t"],
        ];

        for input in inputs.into_iter() {

            in_tmpdir(|tmp_dir| {
                let binary_path = tmp_dir.path().join(input[0]);
                let cur_cmd = String::from(input[1]);
                let cur_path = tmp_dir.path().join(input[2]);
                let expected = input[3];
                
                let cmd_path = cur_path.join(cur_cmd.clone());
                
                ensure_files_created(
                    tmp_dir.path().to_path_buf(),
                    [
                        binary_path.clone(),
                        cmd_path,
                    ].as_ref()
                );
                
                let expectation = match expected {
                    "t" => true,
                    _ => false,
                };
                
                let observed = paths_equivalent(binary_path, cur_cmd, cur_path.as_path()).unwrap();
                assert_eq!(observed, expectation)
            });
        }
    }
}
