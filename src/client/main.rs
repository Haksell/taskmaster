use std::os::unix::net::UnixStream;
use std::io::Write;
use rustmaster_core::UNIX_DOMAIN_SOCKET_PATH;
use std::io::stdin;

fn main() {
    let mut stream = match UnixStream::connect(UNIX_DOMAIN_SOCKET_PATH) {
        Ok(unix_stream) => unix_stream,
        Err(_) => {
            eprintln!("Can't connect to \"{}\"", UNIX_DOMAIN_SOCKET_PATH);
            return;
        }
    };
    let mut buffer = String::new();
    let stdin = stdin();

    while stdin.read_line(&mut buffer).is_ok() {
        let trimmed = buffer.trim();
        stream.write_all(trimmed.as_bytes()).expect("Can't write in socket");
        buffer.clear();
    }
}
