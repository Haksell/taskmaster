use std::os::unix::net::UnixListener;
use std::io::Read;
use rustmaster_core::UNIX_DOMAIN_SOCKET_PATH;

fn main() {
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
                let mut buffer = [0; 128];
                while let Ok(bytes_received) = stream.read(&mut buffer) {
                    if bytes_received == 0 {
                        break;
                    }
                    let received_data = String::from_utf8_lossy(&buffer[..bytes_received]);
                    println!("Received: {}", received_data);
                }
            }
            Err(e) => {
                eprintln!("Error accepting connection: {}", e);
            }
        }
    }
}
