mod api;
mod core;
mod monitor;

use crate::api::Responder;
use crate::core::configuration::Configuration;
use crate::core::logger::Logger;
use crate::monitor::Monitor;
use daemonize::Daemonize;
use std::env;
use std::process::exit;

static mut CONFIG_FILE_NAME: Option<String> = None;
static mut IS_DISABLED_DEMONIZE: bool = false;
const HELP_MESSAGE: &str = "Options are:\n\t--help: Show help info\
    \n\t--debug: Disables daemon mode and shows logs\
    \n\t<path_to_config_file>: Starts server with a configuration";

unsafe fn parse_arguments() {
    let args: Vec<String> = env::args().skip(1).collect();
    for arg in args {
        match arg.as_str() {
            "--help" => {
                eprintln!("{}", HELP_MESSAGE);
                exit(2);
            }
            "--debug" => IS_DISABLED_DEMONIZE = true,
            _ => {
                if CONFIG_FILE_NAME.is_none() {
                    CONFIG_FILE_NAME = Some(arg.clone());
                } else {
                    eprintln!(
                        "Error: Config file path is already defined: {}. What is {}?",
                        CONFIG_FILE_NAME.clone().unwrap(),
                        arg
                    );
                    exit(1);
                }
            }
        }
    }
}

fn run_program(monitor: &mut Monitor) {
    monitor.track();
    Responder::listen(monitor);
}

fn main() {
    let mut logger = Logger::new(None);
    let mut monitor: Monitor;
    unsafe {
        parse_arguments();
        if IS_DISABLED_DEMONIZE {
            logger = Logger::enable(None);
            logger.log("The logging was enabled")
        }
        monitor = Monitor::new();
        if let Some(file_name) = CONFIG_FILE_NAME.clone() {
            logger.log(format!(
                "Configuration file [{}] was provided on start",
                file_name
            ));
            match Configuration::from_yml(String::from(file_name)) {
                Ok(conf) => {
                    monitor.update_configuration(conf);
                }
                Err(err_msg) => {
                    logger.log_err(err_msg);
                    exit(2);
                }
            }
        } else {
            logger.log("No configuration file was provided")
        }
    }

    let daemon = Daemonize::new()
        .pid_file("/var/run/server.pid")
        .chown_pid_file(true)
        .working_directory(".");
    unsafe {
        if IS_DISABLED_DEMONIZE {
            run_program(&mut monitor);
        } else {
            match daemon.start() {
                Ok(_) => {
                    run_program(&mut monitor);
                }
                Err(e) => {
                    eprintln!("Can't daemonize {e}. Already launched or check sudo")
                }
            }
        }
    }
}
