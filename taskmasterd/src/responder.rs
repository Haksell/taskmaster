use crate::action::Action;
use crate::logger::Logger;
use crate::monitor::Monitor;
use crate::responder::Respond::Message;
use crate::{remove_and_exit, UNIX_DOMAIN_SOCKET_PATH};
use libc::printf;
use std::borrow::Cow;
use std::collections::VecDeque;
use std::io::{Read, Write};
use std::os::unix::fs::PermissionsExt;
use std::os::unix::net::{UnixListener, UnixStream};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use std::{fs, thread};

pub enum Respond {
    Message(String),
    MaintailStream,
    TailStream(String),
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

    fn write_message(&mut self, mut stream: UnixStream, respond: Respond) {
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
            Respond::MaintailStream => {
                let logger_clone = self.logger.clone();
                thread::spawn(move || {
                    let mut history_buffer: VecDeque<String> = {
                        let logger = logger_clone.lock().unwrap();
                        logger.history.clone()
                    };
                    let mut last_logged_message: String = String::new();

                    'outer: loop {
                        let logger = logger_clone.lock().unwrap();

                        while !history_buffer.is_empty() {
                            if let Some(message) = history_buffer.pop_front() {
                                last_logged_message = message;
                                if let Err(err) = stream.write(last_logged_message.as_bytes()) {
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
                        let to_append: Vec<String> = history
                            .iter()
                            .skip_while(|elem| **elem != last_logged_message)
                            .cloned()
                            .collect();
                        for element in to_append.iter() {
                            if *element != last_logged_message {
                                history_buffer.push_back(element.clone());
                            }
                        }

                        drop(logger);

                        thread::sleep(Duration::from_millis(150));
                    }
                });
            }

            Respond::TailStream(_) => {}
        }
        /*
        loop {
            stream.write(b"sa");
            stream.flush();
            thread::sleep(Duration::from_secs(1));
        }
         */
    }

    fn handle_message(&mut self, stream: UnixStream, received_data: Cow<str>) {
        {
            let mut logger = self.logger.lock().unwrap();
            logger.resp_log(format!("Received via socket: {received_data}"));
        }
        match serde_json::from_str::<Action>(received_data.to_string().as_str()) {
            Ok(action) => {
                let answer = self.monitor.answer(action);
                self.write_message(stream, answer);
            }
            Err(error) => {
                {
                    let mut logger = self.logger.lock().unwrap();
                    logger.resp_log(format!("Unknown action: {received_data}: {error}"));
                }
                self.write_message(stream, Message("Unknown action".to_string()));
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
