use rustmaster_core::api::Action;
use rustmaster_core::api::Action::Exit;
use rustmaster_core::UNIX_DOMAIN_SOCKET_PATH;
use std::io::stdin;
use std::io::{stdout, Read, Write};
use std::os::unix::net::UnixStream;

//TODO: clean up, handle errors
fn main() {
    loop {
        let mut stream = match UnixStream::connect(UNIX_DOMAIN_SOCKET_PATH) {
            Ok(unix_stream) => unix_stream,
            Err(_) => {
                eprintln!("Can't connect to \"{}\"", UNIX_DOMAIN_SOCKET_PATH);
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
        if trimmed == Exit.to_string() {
            break;
        }

        match Action::from(trimmed) {
            Ok(action) => {
                let serialized_action =
                    serde_json::to_string(&action).expect("Serialization failed");
                stream.write_all(serialized_action.as_bytes()).expect("aa");

                let mut response = String::new();
                stream.read_to_string(&mut response).expect("aaa");
                println!("{}", response);
            }
            Err(err) => {
                println!("{}", err);
            }
        }
    }
}
