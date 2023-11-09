use crate::action::Action;
use crate::logger::{LogLine, Logger};
use crate::monitor::Monitor;
use crate::responder::Respond::Message;
use crate::{remove_and_exit, UNIX_DOMAIN_SOCKET_PATH};
use std::borrow::Cow;
use std::collections::VecDeque;
use std::io::{stdout, Read, Seek, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

pub enum Respond {
    Message(String),
    MaintailStream(Option<usize>),
    Tail(String, Option<usize>, bool),
}

pub struct Responder {
    logger: Arc<Mutex<Logger>>,
    monitor: Monitor,
}

impl Responder {
    fn bind_listener(&self) -> UnixListener {
        let mut logger = self.logger.lock().unwrap();
        return match UnixListener::bind(UNIX_DOMAIN_SOCKET_PATH) {
            Ok(stream) => {
                if let Err(_) =
                    fs::set_permissions(UNIX_DOMAIN_SOCKET_PATH, fs::Permissions::from_mode(0o666))
                {
                    logger.log_err(format!(
                        "Can't change permissions of \"{UNIX_DOMAIN_SOCKET_PATH}\""
                    ));
                    remove_and_exit(2);
                }
                logger.log(format!(
                    "Socket was successfully created: {UNIX_DOMAIN_SOCKET_PATH}"
                ));
                stream
            }
            Err(_) => {
                logger.log_err(format!("Can't bind socket \"{UNIX_DOMAIN_SOCKET_PATH}\""));
                remove_and_exit(2);
            }
        };
    }

    //TODO: fn write()

    fn handle_response(&mut self, mut stream: UnixStream, respond: Respond) {
        let mut logger = self.logger.lock().unwrap();
        match respond {
            Message(message) => {
                if let Err(e) = stream.write(message.as_bytes()) {
                    logger.resp_log(format!(
                        "Can't answer to the client with message: \"{message}\": {e}"
                    ));
                } else {
                    logger.resp_log(format!("Sending the answer: \"{message}\""));
                }
                if let Err(e) = stream.flush() {
                    logger.resp_log(format!("Can't flush the stdout: {e}"));
                }
            }
            Respond::MaintailStream(num_lines) => {
                let logger_clone = self.logger.clone();
                thread::spawn(move || {
                    let mut history_buffer: VecDeque<LogLine> = {
                        let logger = logger_clone.lock().unwrap();
                        if let Some(num_lines) = num_lines {
                            logger
                                .history
                                .iter()
                                .rev()
                                .take(num_lines)
                                .rev()
                                .cloned()
                                .collect()
                        } else {
                            logger.history.clone()
                        }
                    };
                    let mut last_logged_idx = 0usize;

                    'outer: loop {
                        let logger = logger_clone.lock().unwrap();

                        while !history_buffer.is_empty() {
                            if let Some((idx, message)) = history_buffer.pop_front() {
                                last_logged_idx = idx;
                                if let Err(err) = stream.write(message.as_bytes()) {
                                    eprintln!("Exiting maintail -f: {:?}", err);
                                    break 'outer;
                                }
                            }
                            if let Err(err) = stream.flush() {
                                eprintln!("Exiting maintail -f: {:?}", err);
                                break 'outer;
                            }
                        }

                        let history = logger.history.clone();
                        let to_append: Vec<_> = history
                            .iter()
                            .skip_while(|(idx, _)| *idx != last_logged_idx)
                            .skip(1)
                            .cloned()
                            .collect();
                        for element in to_append.iter() {
                            history_buffer.push_back(element.clone());
                        }

                        drop(logger);
                        thread::sleep(Duration::from_millis(100));
                    }
                });
            }
            Respond::Tail(filename, num_lines, is_stream) => {
                match fs::File::open(&filename) {
                    Ok(mut file) => {
                        let mut buffer = String::new();
                        match file.read_to_string(&mut buffer) {
                            Ok(_) => {
                                print!("{buffer}");
                                stdout().flush().unwrap(); //FIXME: handle
                                match num_lines {
                                    None => {
                                        stream.write(buffer.as_bytes()).unwrap();
                                    } //FIXME: handle,
                                    Some(num) => {
                                        let mut lines: Vec<String> = buffer
                                            .lines()
                                            .rev()
                                            .take(num)
                                            .map(String::from)
                                            .collect();
                                        lines = lines.iter().cloned().rev().collect();
                                        stream
                                            .write(
                                                lines
                                                    .iter()
                                                    .map(|line| line.to_string() + "\n")
                                                    .collect::<String>()
                                                    .as_bytes(),
                                            )
                                            .unwrap(); //TODO: handle
                                        stream.flush().unwrap(); //TODO: handle
                                    }
                                }
                                if is_stream {
                                    let mut last_size = fs::metadata(&filename).unwrap().len(); //TODO: handle
                                    thread::spawn(move || loop {
                                        thread::sleep(Duration::from_millis(100));
                                        let new_size = fs::metadata(&filename).unwrap().len(); //TODO: handle
                                        println!("{last_size} -> {new_size}");
                                        if new_size < last_size {
                                            if let Err(e) = stream.write(
                                                format!("\n\ntail: {filename}: file truncated\n\n")
                                                    .as_bytes(),
                                            ) {
                                                eprintln!("{e}");
                                                break;
                                            }
                                            stream.flush().unwrap(); //TODO: handle
                                            file.seek(std::io::SeekFrom::Start(0)).unwrap();
                                        } else if new_size == last_size {
                                            continue;
                                        }
                                        let mut new_content = String::new();
                                        file.read_to_string(&mut new_content).unwrap(); //TODO: handle
                                        println!("{}", new_content.len());
                                        if let Err(e) = stream.write(new_content.as_bytes()) {
                                            eprintln!("{e}");
                                            break;
                                        }
                                        stream.flush().unwrap();
                                        last_size = new_size;
                                    });
                                }
                            }
                            Err(_) => {
                                eprintln!("Can't read file")
                            }
                        }
                    }
                    Err(_) => {
                        eprintln!("Can't open file")
                    }
                }
            }
        }
    }

    fn handle_message(&mut self, stream: UnixStream, received_data: Cow<str>) {
        {
            let mut logger = self.logger.lock().unwrap();
            logger.resp_log(format!("Received via socket: {received_data}"));
        }
        match serde_json::from_str::<Action>(received_data.to_string().as_str()) {
            Ok(action) => {
                let answer = self.monitor.answer(action);
                self.handle_response(stream, answer);
            }
            Err(error) => {
                {
                    let mut logger = self.logger.lock().unwrap();
                    logger.resp_log(format!("Unknown action: {received_data}: {error}"));
                }
                self.handle_response(stream, Message("Unknown action".to_string()));
            }
        }
    }

    pub fn listen(monitor: Monitor, logger: Arc<Mutex<Logger>>) {
        let mut responder = Responder { logger, monitor };
        for stream in responder.bind_listener().incoming() {
            match stream {
                Ok(mut stream) => {
                    let mut buffer = [0; 1024];
                    match stream.read(&mut buffer) {
                        Ok(bytes_read) => {
                            if bytes_read == 0 {
                                continue;
                            }
                            responder.handle_message(
                                stream,
                                String::from_utf8_lossy(&buffer[..bytes_read]),
                            );
                        }
                        Err(e) => {
                            let mut logger = responder.logger.lock().unwrap();
                            logger.resp_log(format!("Stream: {e}"));
                        }
                    }
                }
                Err(e) => {
                    let logger = responder.logger.lock().unwrap();
                    logger.log_err(format!("Can't accept a connection: {e}"));
                }
            }
        }
    }
}
