mod action;
mod configuration;
mod logger;
mod monitor;
mod responder;
mod sighup_handler;
mod task;
mod utils;

use configuration::Configuration;
use daemonize::Daemonize;
use logger::Logger;
use monitor::Monitor;
use responder::Responder;
use std::env;
use std::sync::{Arc, Mutex};

pub const UNIX_DOMAIN_SOCKET_PATH: &'static str = "/tmp/taskmaster.sock";
pub const PID_FILE_PATH: &'static str = "/tmp/taskmasterd.pid";
pub const LOG_FILE_PATH: &'static str = "/tmp/taskmasterd.log";

const HELP_MESSAGE: &str = "Options are:\n\t--help: Show help info\
    \n\t--debug: Disables daemon mode\
    \n\t<path_to_config_file>: Starts server with a configuration";

macro_rules! error_exit {
    ($code:expr, $($arg:tt)*) => {
        {
            eprintln!($($arg)*);
            remove_and_exit($code);
        }
    };
}

fn remove_files() {
    let _ = std::fs::remove_file(UNIX_DOMAIN_SOCKET_PATH);
    let _ = std::fs::remove_file(PID_FILE_PATH);
    let _ = std::fs::remove_file(LOG_FILE_PATH);
}

pub fn remove_and_exit(exit_code: i32) -> ! {
    remove_files();
    std::process::exit(exit_code);
}

fn check_root_user() {
    let euid = unsafe { libc::geteuid() };
    if euid != 0 {
        error_exit!(
            1,
            "Error: taskmasterd must be run as the root user in non-debug mode."
        );
    }
}

fn parse_arguments() -> (bool, String) {
    let mut should_daemonize = true;
    let mut filename: Option<String> = None;
    for arg in env::args().skip(1) {
        match arg.as_str() {
            "--help" => {
                println!("{}", HELP_MESSAGE);
                remove_and_exit(0);
            }
            "--debug" => should_daemonize = false,
            _ => {
                if arg.starts_with("-") {
                    error_exit!(2, "Error: Unknown option: {arg}");
                } else {
                    match filename {
                        None => filename = Some(arg.clone()),
                        Some(old) =>  error_exit!(
                            2,
                            "Error: Configuration file is already defined: \"{old}\". What is \"{arg}\"?",
                        )
                    }
                }
            }
        }
    }
    match filename {
        Some(filename) => (should_daemonize, filename),
        None => error_exit!(2, "Error: No configuration file given"),
    }
}

fn run_program(monitor: Monitor, logger: Arc<Mutex<Logger>>) {
    monitor.track();
    Responder::listen(monitor, logger);
}

fn main() {
    remove_files();
    let (should_daemonize, config_path) = parse_arguments();
    sighup_handler::set_sighup_handler();

    match Logger::new(LOG_FILE_PATH) {
        Ok(logger) => {
            let logger = Arc::new(Mutex::new(logger));
            println!("taskmasterd launched (PID {})", std::process::id());

            let mut monitor = Monitor::new(config_path.clone(), logger.clone());
            match Configuration::from_yml(config_path, logger.clone()) {
                Ok(conf) => {
                    monitor.update_configuration(conf);
                }
                Err(err_msg) => {
                    error_exit!(2, "{err_msg}");
                }
            }

            if should_daemonize {
                check_root_user();
                match Daemonize::new()
                    .pid_file(PID_FILE_PATH)
                    .chown_pid_file(true)
                    .working_directory(".")
                    .user("nobody")
                    .group("daemon")
                    .umask(0o022)
                    .start()
                {
                    Ok(_) => run_program(monitor, logger),
                    Err(e) => eprintln!("Can't daemonize: {e}. Already launched or check sudo"),
                }
            } else {
                run_program(monitor, logger);
            }
        }
        Err(error) => {
            error_exit!(1, "{error}")
        }
    }
    remove_files();
}
