use std::env;
use std::time::Instant;
use std::process::{exit, Command};
#[macro_use] extern crate log;
extern crate env_logger;
extern crate hull;


fn abort(exit_code :i32, message :String) -> ! {
    match hull::cmd::stderr_write(message) {
        Ok(_) => exit(exit_code),
        Err(e) => unexpected_io_error(e),
    };
}


fn unexpected_io_error(err :std::io::Error) -> ! {
    println!("failure: {}", err.to_string());
    exit(1);
}


fn main() {
    let now = Instant::now();
    env_logger::init().unwrap_or_else(|err| {
        abort(1, format!("Error: {}", err));
    });

    let args: Vec<String> = env::args().collect();

    let ref cmd = args[0];
    let cmd_args = &args[1 .. args.len()];

    // remove binary path from PATH so the shell can resolve the next command location
    let binary_path = match env::current_exe() {
        Ok(res) => res,
        Err(e) => {
            abort(1, format!("Error: unable to get executable path\n{}", e));
        },
    };

    let binary_dir = String::from(
        binary_path.parent().unwrap()
            .to_string_lossy()
            .into_owned()
    );

    // current shell invocation path
    let cur_dir = match env::current_dir() {
        Ok(res) => res,
        Err(e) => abort(1, format!("Error: cannot get current directory\n{}", e)),
    };
    let cur_path = cur_dir.as_path();

    match hull::cmd::paths_equivalent(binary_path, cmd.clone(), cur_path) {
        Ok(equivalent) => {
            if equivalent {
                abort(1,
                      String::from("Error: unwilling to run program recursively. ")
                      + "Please check your paths",
                );
            };
        },
        Err(e) => abort(1, format!("Error: cannot open paths\n{}", e.to_string())),
    };

    debug!("command: '{}' args: '{:#?}'", cmd, cmd_args);

    let path_str = match env::var("PATH") {
        Ok(res) => res,
        Err(e) => abort(1, format!("Error: cannot get PATH environment variable\n{}", e)),
    };

    let new_path_str = hull::cmd::remove_dir_from_path(binary_dir, path_str);

    let boot_duration = now.elapsed();
    let run_start = Instant::now();

    let result = Command::new(cmd)
        .args(cmd_args)
        .env("PATH", new_path_str)
        .current_dir(cur_path)
        .status();

    let run_duration = run_start.elapsed();

    let status_code = match result {
        // exit for all statuses except expected ones

        Ok(status) => {
            match status.code() {
                Some(status_code) => status_code,
                None => abort(1, format!("{} : unknown error running command", cmd)),
            }
        },
        Err(e) => {
            match e.raw_os_error() {
                Some(error_code) => abort(error_code, format!("{} : {}", cmd, e.to_string())),
                None => abort(1, format!("{} : unknown error running command", cmd)),
            };
        }
    };

    let total_duration = now.elapsed();

    let boot_time = hull::cmd::duration_in_millis(boot_duration);
    let run_time = hull::cmd::duration_in_millis(run_duration);
    let total_time = hull::cmd::duration_in_millis(total_duration);
    info!("boot_time: {} ms run_time:{ } ms total_time: {} ms", boot_time, run_time, total_time);

    exit(status_code);
}
