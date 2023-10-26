mod api;
mod core;
mod monitor;

use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::env;
use std::process::exit;
use crate::api::Action;
use crate::core::configuration::Configuration;
use crate::core::task::UNIX_DOMAIN_SOCKET_PATH;
use crate::monitor::Monitor;


fn answer(monitor: &mut Monitor, action: Action) -> String {
    match action {
        Action::Status(status) => monitor.get_task_status(status),
        Action::Help => action.get_description(),
        Action::Config(task_name) => {
            match monitor.get_task_by_name(&task_name) {
                None => format!("Can't find \"{}\" task", task_name),
                Some(task) => format!("{}: {}", task_name, task.get_json_configuration()),
            }
        }
        Action::Exit => String::new(),
        Action::Start(task_name) => match monitor.start_task_by_name(&task_name) {
            Ok(_) => String::new(),
            Err(err_msg) => err_msg,
        },
        Action::Stop(task_name) => match monitor.stop_task_by_name(&task_name) {
            Ok(_) => String::new(),
            Err(err_msg) => err_msg,
        },
    }
}


//TODO: clean up, separate to different fn, handle errors

fn get_config_file_name() -> Option<String> {
    let args: Vec<String> = env::args().collect();
    if args.len() > 2 {
        eprintln!("Usage: server\n\tserver <path_to_config_file>");
        exit(2);
    } else if args.len() == 1 {
        return None;
    }
    Some(args.get(1).unwrap().clone())
}

fn main() {
    let mut monitor = Monitor::new();
    if let Some(config_file_name) = get_config_file_name() {
        match Configuration::from_yml(String::from(config_file_name)) {
            Ok(conf) => monitor.load_configuration(conf),
            Err(err_msg) => {
                eprintln!("{err_msg}");
                exit(2);
            }
        }
    }

    monitor.run_autostart();

    let listener = match UnixListener::bind(UNIX_DOMAIN_SOCKET_PATH) {
        Ok(stream) => stream,
        Err(_) => {
            eprintln!("Can't listen \"{}\"", UNIX_DOMAIN_SOCKET_PATH);
            exit(2);
        }
    };
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buffer = [0; 1024];
                let bytes_read = stream.read(&mut buffer).unwrap();
                if bytes_read == 0 {
                    continue;
                }
                let received_data = String::from_utf8_lossy(&buffer[..bytes_read]);
                println!("Received: {}", received_data);
                let received_action =
                    serde_json::from_str::<Action>(received_data.to_string().as_str()).unwrap();
                stream
                    .write(answer(&mut monitor, received_action).as_bytes())
                    .unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
