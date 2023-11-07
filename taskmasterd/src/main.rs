mod api;
mod core;
mod monitor;
mod sighup_handler;

use crate::api::Responder;
use crate::core::configuration::Configuration;
use crate::core::logger::Logger;
use crate::monitor::Monitor;
use daemonize::Daemonize;
use std::env;

pub const UNIX_DOMAIN_SOCKET_PATH: &'static str = "/run/taskmaster.sock";
pub const PID_FILE_PATH: &'static str = "/run/taskmasterd.pid";

const HELP_MESSAGE: &str = "Options are:\n\t--help: Show help info\
    \n\t--no-daemon: Disables daemon mode\
    \n\t<path_to_config_file>: Starts server with a configuration";

macro_rules! error_exit {
    ($($arg:tt)*) => {
        {
            eprintln!($($arg)*);
            remove_and_exit(2);
        }
    };
}

fn remove_files() {
    let _ = std::fs::remove_file(UNIX_DOMAIN_SOCKET_PATH);
    let _ = std::fs::remove_file(PID_FILE_PATH);
}

pub fn remove_and_exit(exit_code: i32) -> ! {
    remove_files();
    std::process::exit(exit_code);
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
            "--no-daemonize" => should_daemonize = false,
            _ => {
                if arg.starts_with("-") {
                    error_exit!("Error: Unknown option: {arg}");
                } else if filename.is_none() {
                    filename = Some(arg.clone());
                } else {
                    error_exit!(
                        "Error: Configuration file is already defined: \"{}\". What is \"{}\"?",
                        filename.unwrap(),
                        arg
                    );
                }
            }
        }
    }
    if filename.is_none() {
        error_exit!("Error: No configuration file given");
    }
    (should_daemonize, filename.unwrap())
}

fn run_program(monitor: &mut Monitor) {
    monitor.track();
    Responder::listen(monitor);
}

fn main() {
    remove_files();
    let (should_daemonize, config_path) = parse_arguments();
    sighup_handler::set_sighup_handler();

    let logger = Logger::new(None);
    logger.log(format!("taskmasterd launched (PID {})", std::process::id()));

    let mut monitor = Monitor::new(config_path.clone());
    match Configuration::from_yml(config_path) {
        Ok(conf) => {
            monitor.update_configuration(conf);
        }
        Err(err_msg) => {
            logger.log_err(err_msg);
            remove_and_exit(2);
        }
    }

    if should_daemonize {
        match Daemonize::new()
            .pid_file(PID_FILE_PATH)
            .chown_pid_file(true)
            .working_directory(".")
            .start()
        {
            Ok(_) => run_program(&mut monitor),
            Err(e) => eprintln!("Can't daemonize: {e}. Already launched or check sudo"),
        }
    } else {
        run_program(&mut monitor);
    }
    remove_files();
}
