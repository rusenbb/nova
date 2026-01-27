//! IPC helpers for GTK frontend.

use std::env;
use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;

pub fn get_socket_path() -> PathBuf {
    let runtime_dir = env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(runtime_dir).join("nova.sock")
}

pub fn try_send_toggle() -> bool {
    let socket_path = get_socket_path();
    if let Ok(mut stream) = UnixStream::connect(&socket_path) {
        let _ = stream.write_all(b"toggle");
        let mut response = [0u8; 2];
        let _ = stream.read_exact(&mut response);
        return true;
    }
    false
}
