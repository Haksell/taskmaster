mod api;
mod core;
mod monitor;

use crate::api::Responder;
use crate::core::configuration::Configuration;
use crate::core::logger::Logger;
use crate::monitor::Monitor;
use std::env;
use std::process::exit;

static mut CONFIG_FILE_NAME: Option<String> = None;
static mut IS_ENABLED_LOGGING: bool = false;
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
            "--debug" => IS_ENABLED_LOGGING = true,
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

//TODO: clean up, separate to different fn, handle errors
fn main() {
    let mut logger = Logger::new();
    let mut monitor: Monitor;
    unsafe {
        parse_arguments();
        if IS_ENABLED_LOGGING {
            logger = Logger::enable();
            logger.log("The logging was enabled")
        }
        monitor = Monitor::new();
        if let Some(file_name) = CONFIG_FILE_NAME.clone() {
            logger.log(format!(
                "Configuration file [{}] was provided on start",
                file_name
            ));
            match Configuration::from_yml(String::from(file_name)) {
                Ok(conf) => monitor.load_configuration(conf),
                Err(err_msg) => {
                    logger.log_err(err_msg);
                    exit(2);
                }
            }
        } else {
            logger.log("No configuration file was provided")
        }
    }

    monitor.track();
    monitor.run_autostart();

    Responder::listen(&mut monitor);
}
