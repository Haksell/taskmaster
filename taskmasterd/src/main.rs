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

const HELP_MESSAGE: &str = "Options are:\n\t--help: Show help info\
    \n\t--debug: Disables daemon mode and shows logs\
    \n\t<path_to_config_file>: Starts server with a configuration";

fn parse_arguments() -> (bool, Option<String>) {
    let mut result = (false, None);
    let args: Vec<String> = env::args().skip(1).collect();
    for arg in args {
        match arg.as_str() {
            "--help" => {
                eprintln!("{}", HELP_MESSAGE);
                exit(2);
            }
            "--debug" => result.0 = true,
            _ => {
                if result.1.is_none() {
                    result.1 = Some(arg.clone());
                } else {
                    eprintln!(
                        "Error: Config file path is already defined: {}. What is {}?",
                        result.1.unwrap(),
                        arg
                    );
                    exit(1);
                }
            }
        }
    }
    result
}

fn run_program(monitor: &mut Monitor, main_logger: &Logger) {
    main_logger.log("taskmasterd launched");
    monitor.track();
    Responder::listen(monitor);
}

fn main() {
    let logger = Logger::new(None);
    let parsed_args = parse_arguments();
    let is_disabled_demonize = parsed_args.0;
    let config_file_name = parsed_args.1;
    let mut monitor = Monitor::new();
    if let Some(file_name) = config_file_name {
        match Configuration::from_yml(String::from(file_name)) {
            Ok(conf) => {
                monitor.update_configuration(conf);
            }
            Err(err_msg) => {
                logger.log_err(err_msg);
                exit(2);
            }
        }
    }

    if is_disabled_demonize {
        run_program(&mut monitor, &logger);
    } else {
        match Daemonize::new()
            .pid_file("/var/run/server.pid")
            .chown_pid_file(true)
            .working_directory(".")
            .start()
        {
            Ok(_) => run_program(&mut monitor, &logger),
            Err(e) => eprintln!("Can't daemonize: {e}. Already launched or check sudo"),
        }
    }
}
