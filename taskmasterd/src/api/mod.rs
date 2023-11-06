use crate::api::action::Action;
use crate::core::logger::Logger;
use crate::monitor::Monitor;
use std::borrow::Cow;
use std::io::{Read, Write};
use std::os::unix::net::{UnixListener, UnixStream};
use std::process::exit;

pub mod action;

pub const UNIX_DOMAIN_SOCKET_PATH: &str = "/tmp/.unixdomain.sock";

pub struct Responder<'a> {
    logger: &'a Logger,
    stream: UnixStream,
    monitor: &'a mut Monitor,
}

impl<'a> Responder<'a> {
    fn bind_listener() -> UnixListener {
        let logger = Logger::new(Some("Responder"));
        return match UnixListener::bind(UNIX_DOMAIN_SOCKET_PATH) {
            Ok(stream) => {
                logger.log(format!(
                    "Socket was successfully created: {UNIX_DOMAIN_SOCKET_PATH}"
                ));
                stream
            }
            Err(_) => {
                logger.log_err(format!(
                    "Error! Can't bind socket \"{UNIX_DOMAIN_SOCKET_PATH}\""
                ));
                exit(2);
            }
        };
    }

    fn write_message(&mut self, message: &String) {
        if let Err(e) = self.stream.write(message.as_bytes()) {
            self.logger.log(format!(
                "Error! Can't answer to the client with message: \"{message}\": {e}"
            ));
        }
        if let Err(e) = self.stream.flush() {
            self.logger
                .log(format!("Error! Can't flush the stdout: {e}"));
        }
    }

    fn handle_message(&mut self, received_data: Cow<str>) {
        self.logger
            .log(format!("Received via socket: {received_data}"));

        match serde_json::from_str::<Action>(received_data.to_string().as_str()) {
            Ok(action) => {
                let answer = self.monitor.answer(action);
                self.write_message(&answer);
                self.logger.log(format!("Sending the answer: \"{answer}\""));
            }
            Err(error) => {
                self.logger
                    .log(format!("Error! Unknown action: {received_data}: {error}"));
                self.write_message(&"Error! Unknown action".to_string());
            }
        }
    }

    pub fn listen(monitor: &mut Monitor) {
        let logger = Logger::new(Some("Responder"));
        for stream in Responder::bind_listener().incoming() {
            match stream {
                Ok(stream) => {
                    let mut listener = Responder {
                        logger: &logger,
                        stream,
                        monitor,
                    };
                    let mut buffer = [0; 1024];
                    match listener.stream.read(&mut buffer) {
                        Ok(bytes_read) => {
                            if bytes_read == 0 {
                                continue;
                            }
                            listener.handle_message(String::from_utf8_lossy(&buffer[..bytes_read]));
                        }
                        Err(e) => {
                            listener.logger.log(e.to_string());
                        }
                    }
                }
                Err(e) => {
                    logger.log_err(format!("Error! Can't accept a connection: {e}"));
                }
            }
        }
    }
}
