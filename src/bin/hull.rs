#[macro_use]
extern crate log;
#[macro_use]
extern crate crossbeam_channel;
extern crate env_logger;
extern crate hull;
extern crate time;

use hull::cmd::{path, process, signal, utils};
use hull::types::{CommandRunTime, Event, EventsPayload};
use std::env;
use std::process::exit;
use std::time::Instant;
use time::get_time;

fn main() {
    let now = Instant::now();
    env_logger::init();

    let mut signal_handler = signal::SignalHandler::new();
    signal_handler.handle_in_thread().unwrap_or_else({
        |e| utils::abort(1, format!("Error: cannot setup signal handler\n{}", e))
    });

    let args: Vec<String> = env::args().collect();

    let ref cmd = args[0];
    let cmd_args = &args[1..args.len()];

    let symlinks_path = path::get_hull_symlinks_root();
    if !symlinks_path.exists() {
        utils::abort(
            -1,
            String::from("Error: cannot find hull symlinks directory. Please set HULL_ROOT"),
        );
    }

    let symlinks_dir = String::from(symlinks_path.to_string_lossy().as_ref());

    // current shell invocation path
    let cur_dir = match env::current_dir() {
        Ok(res) => res,
        Err(e) => utils::abort(1, format!("Error: cannot get current directory\n{}", e)),
    };
    let cur_path = cur_dir.as_path();

    match path::paths_equivalent(symlinks_path, cmd.clone(), cur_path) {
        Ok(equivalent) => {
            if equivalent {
                utils::abort(
                    1,
                    String::from("Error: unwilling to run program recursively. ")
                        + "Please check your paths",
                );
            };
        }
        Err(e) => utils::abort(1, format!("Error: cannot open paths\n{}", e.to_string())),
    };

    debug!("command: '{}' args: '{:#?}'", cmd, cmd_args);

    let path_str = match env::var("PATH") {
        Ok(res) => res,
        Err(e) => utils::abort(
            1,
            format!("Error: cannot get PATH environment variable\n{}", e),
        ),
    };

    // remove symlinks path from PATH so the shell can resolve the next command location
    let new_path_str = path::remove_dir_from_path(symlinks_dir, path_str);

    let run_start = Instant::now();

    let cmd_params = process::CommandParams {
        cmd: cmd.to_string(),
        args: cmd_args.to_vec(),
        path: new_path_str.to_string(),
        dir: cur_path.to_path_buf(),
    };

    let mut process = process::Process::new(cmd_params);
    let pid = process.run_in_thread().unwrap_or_else({
        |e| {
            utils::abort(
                1,
                format!("Error: Could not start program {}, error code: {}", cmd, e),
            )
        }
    });

    let cmd_rx = process.receiver();
    let sig_rx = signal_handler.receiver();
    let status_code;
    loop {
        select! {
            recv(sig_rx) -> sig_result => {
                let sig = sig_result
                    .unwrap_or_else({ |e| utils::abort(1, format!("Error: Cannot trap signal: {}", e))});
                info!("Received signal: {}. Sending to pid: {}", sig, pid);
                signal::kill(pid, sig).unwrap_or_else({
                    |e| utils::abort(1, format!("Error: Cannot send signal: {}", e))
                });
            },
            recv(cmd_rx) -> status_code_result => {
                status_code = status_code_result
                    .unwrap_or_else({ |e| utils::abort(1, format!("Error: Cannot obtain status code: {}", e))});
                break;
            }
        }
    }

    let run_duration = run_start.elapsed();

    let total_duration = now.elapsed();
    let run_time = utils::duration_in_millis(run_duration);

    let telemetry_start = Instant::now();
    utils::update_telemetry(&EventsPayload {
        events: vec![Event {
            event_name: "hull_timing".to_string(),
            event_data: CommandRunTime {
                cmd: cmd.to_string(),
                args: cmd_args.to_vec(),
                run: run_time,
                created_at: get_time().sec,
                status_code,
            },
        }],
    })
    .ok();

    let telemetry_duration = telemetry_start.elapsed();
    let telemetry_time = utils::duration_in_millis(telemetry_duration);

    let total_time = utils::duration_in_millis(total_duration);

    info!(
        "cmd: {} args: {:?} run: {} ms telemetry: {} ms total: {} ms",
        cmd, args, run_time, telemetry_time, total_time
    );

    exit(status_code);
}
