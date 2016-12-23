use std::env;
use std::time::Instant;
use std::process::{exit, Command};
#[macro_use] extern crate log;
extern crate env_logger;
extern crate hull;


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

    if hull::cmd::paths_equivalent(binary_path, cmd.clone(), cur_path) {
        hull::cmd::stderr_write(String::from("Error: Unwilling to run program recursively.")
                                + "Please check your paths."
        ).unwrap();
        exit(1);
    }

    debug!("command: '{}' args: '{:#?}'", cmd, cmd_args);

    let path_str = env::var("PATH").unwrap();
    let new_path_str = hull::cmd::remove_dir_from_path(binary_dir, path_str);

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
        hull::cmd::stderr_write(format!("{} : {}", cmd, err.to_string())).unwrap();
        exit(error_code);
    }

    let total_duration = now.elapsed();

    let boot_time = hull::cmd::duration_in_millis(boot_duration);
    let total_time = hull::cmd::duration_in_millis(total_duration);
    info!("boot_time: {} ms total_time: {} ms", boot_time, total_time);
}
