mod actions_tmp;

use std::io::stdin;
use std::io::{stdout, Read, Write};
use std::os::unix::net::UnixStream;
use crate::actions_tmp::Action;
use crate::actions_tmp::Action::Exit;

//TODO: clean up, handle errors
fn main() {
    loop {
        let mut stream = match UnixStream::connect("/tmp/.unixdomain.sock") {
            Ok(unix_stream) => unix_stream,
            Err(_) => {
                eprintln!("Can't connect to \"{}\"", "/tmp/.unixdomain.sock");
                return;
            }
        };
        let mut buffer = String::new();
        let stdin = stdin();

        print!("taskmaster> ");
        stdout().flush().expect("Can't flush stdout");

        buffer.clear();
        stdin.read_line(&mut buffer).expect("Can;t");

        let trimmed = buffer.trim();

        match Action::from(trimmed) {
            Ok(action) => {
                let serialized_action =
                    serde_json::to_string(&action).expect("Serialization failed");
                stream.write_all(serialized_action.as_bytes()).expect("aa");

                let mut response = String::new();
                stream.read_to_string(&mut response).expect("aaa");
                if response.len() > 0 {
                    println!("{}", response);
                }
                if trimmed == Exit.to_string() {
                    break;
                }
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
}
