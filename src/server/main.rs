use rustmaster_core::api::Action;
use rustmaster_core::data::Configuration;
use rustmaster_core::{Task, UNIX_DOMAIN_SOCKET_PATH};
use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;

fn answer(tasks: &BTreeMap<String, Task>, action: Action) -> String {
    return match action {
        Action::Status(status) => {
            return match status {
                None => {
                    let mut result = String::new();
                    for (i, (name, task)) in tasks.into_iter().enumerate() {
                        result += format!("{}: {}", name, task).as_str();
                        if i != tasks.len() - 1 {
                            result += "\n"
                        }
                    }
                    result
                }
                Some(task_name) => match tasks.get(task_name.as_str()) {
                    None => format!("Can't find \"{}\" task", task_name),
                    Some(task) => format!("{}: {}", task_name, task.to_string()),
                },
            };
        }
        Action::Help => action.get_description(),
        Action::Config(task_name) => {
            return match tasks.get(task_name.as_str()) {
                None => format!("Can't find \"{}\" task", task_name),
                Some(task) => format!("{}: {}", task_name, task.get_json_configuration()),
            }
        }
        Action::Exit => String::new(),
    };
}

//TODO: clean up, separate to different fn, handle errors
fn main() {
    let config = Configuration::from_yml(String::from("config_files/main.yml")).unwrap();
    println!("{:?}", config);

    let mut tasks: BTreeMap<String, Task> = config
        .iter()
        .map(|(key, value)| (key.clone(), Task::new(value.clone())))
        .collect();
    drop(config);

    for (_, task) in &mut tasks {
        task.run(false);
    }

    let listener = match UnixListener::bind(UNIX_DOMAIN_SOCKET_PATH) {
        Ok(stream) => stream,
        Err(_) => {
            eprintln!("Can't listen \"{}\"", UNIX_DOMAIN_SOCKET_PATH);
            return;
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
                let recieved_action =
                    serde_json::from_str::<Action>(received_data.to_string().as_str()).unwrap();
                stream
                    .write(answer(&tasks, recieved_action).as_bytes())
                    .unwrap();
                stream.flush().unwrap();
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
