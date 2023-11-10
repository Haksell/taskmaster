use crate::action::Action;
use crate::{remove_and_exit, UNIX_DOMAIN_SOCKET_PATH};
use libc::{sighandler_t, signal, SIGHUP};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

static SIGHUP_RECEIVED: AtomicBool = AtomicBool::new(false);

extern "C" fn handle_sighup(_: libc::c_int) {
    SIGHUP_RECEIVED.store(true, Ordering::SeqCst);
}

fn send_update_message() -> Result<(), Box<dyn std::error::Error>> {
    use std::io::prelude::*;
    use std::os::unix::net::UnixStream;

    let mut stream = UnixStream::connect(UNIX_DOMAIN_SOCKET_PATH)?;
    let serialized_action = serde_json::to_string(&Action::Update(None))?;
    stream.write_all(serialized_action.as_bytes())?;
    Ok(())
}

pub fn set_sighup_handler() {
    thread::spawn(|| loop {
        if SIGHUP_RECEIVED.load(Ordering::SeqCst) {
            if let Err(e) = send_update_message() {
                eprintln!("Failed to send update message: {}", e);
            }
            SIGHUP_RECEIVED.store(false, Ordering::SeqCst);
        }
        thread::sleep(std::time::Duration::from_millis(100));
    });

    unsafe {
        if signal(SIGHUP, handle_sighup as sighandler_t) == libc::SIG_ERR {
            eprintln!("Error setting up signal handler for SIGHUP");
            remove_and_exit(1);
        }
    }
}
