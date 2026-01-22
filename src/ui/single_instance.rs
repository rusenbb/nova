//! Single instance management for Nova.
//!
//! Ensures only one instance of Nova runs at a time using interprocess
//! local sockets. When a second instance starts, it sends a message to
//! the first instance to show itself, then exits.

use interprocess::local_socket::{traits::ListenerExt, GenericFilePath, ListenerOptions, ToFsName};
use std::io::{BufRead, BufReader, Write};
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// The socket name for Nova IPC.
const SOCKET_NAME: &str = "nova-launcher.sock";

/// Messages from other Nova instances.
#[derive(Debug, Clone)]
pub enum InstanceMessage {
    /// Another instance requested to show the window.
    ShowRequested,
}

/// Result of trying to become the primary instance.
pub enum InstanceResult {
    /// We are the primary instance. The receiver will get messages from other instances.
    Primary(Receiver<InstanceMessage>),
    /// Another instance is already running. We sent it a show request.
    Secondary,
}

/// Try to become the primary instance or signal an existing one.
pub fn acquire_or_signal() -> InstanceResult {
    // First, try to connect to an existing instance
    if signal_existing_instance() {
        return InstanceResult::Secondary;
    }

    // No existing instance, try to become the primary
    match start_listener() {
        Some(rx) => InstanceResult::Primary(rx),
        None => {
            // Failed to start listener, maybe race condition - try signaling again
            if signal_existing_instance() {
                InstanceResult::Secondary
            } else {
                // Really can't do anything, just proceed (might have duplicate instances)
                eprintln!("[Nova] Warning: Could not acquire single-instance lock");
                let (_, rx) = mpsc::channel();
                InstanceResult::Primary(rx)
            }
        }
    }
}

/// Get the socket path for the current platform.
fn get_socket_path() -> String {
    #[cfg(target_os = "windows")]
    {
        format!("@{}", SOCKET_NAME)
    }
    #[cfg(not(target_os = "windows"))]
    {
        // Use runtime directory on Unix
        if let Some(runtime_dir) = dirs::runtime_dir() {
            runtime_dir.join(SOCKET_NAME).to_string_lossy().into_owned()
        } else if let Some(cache_dir) = dirs::cache_dir() {
            cache_dir.join(SOCKET_NAME).to_string_lossy().into_owned()
        } else {
            format!("/tmp/{}", SOCKET_NAME)
        }
    }
}

/// Try to signal an existing instance to show.
fn signal_existing_instance() -> bool {
    use interprocess::local_socket::{traits::Stream as _, GenericFilePath, Stream};

    let socket_path = get_socket_path();
    let name = socket_path.to_fs_name::<GenericFilePath>().ok();

    let name = match name {
        Some(n) => n,
        None => return false,
    };

    match Stream::connect(name) {
        Ok(mut stream) => {
            // Send show command
            if stream.write_all(b"show\n").is_ok() {
                println!("[Nova] Signaled existing instance to show");
                true
            } else {
                false
            }
        }
        Err(_) => false,
    }
}

/// Start the IPC listener for receiving commands from other instances.
fn start_listener() -> Option<Receiver<InstanceMessage>> {
    let socket_path = get_socket_path();

    // Clean up old socket file if it exists (Unix only)
    #[cfg(not(target_os = "windows"))]
    {
        let _ = std::fs::remove_file(&socket_path);
    }

    let name = socket_path.clone().to_fs_name::<GenericFilePath>().ok()?;
    let listener = ListenerOptions::new().name(name).create_sync().ok()?;

    let (tx, rx) = mpsc::channel();

    // Spawn listener thread
    thread::spawn(move || {
        listener_loop(listener, tx);
    });

    println!(
        "[Nova] Single instance listener started at: {}",
        socket_path
    );
    Some(rx)
}

/// The main listener loop (runs in background thread).
fn listener_loop(listener: interprocess::local_socket::Listener, tx: Sender<InstanceMessage>) {
    for conn in listener.incoming().filter_map(|c| c.ok()) {
        let tx = tx.clone();
        thread::spawn(move || {
            let reader = BufReader::new(conn);
            for line in reader.lines().map_while(Result::ok) {
                match line.trim() {
                    "show" => {
                        let _ = tx.send(InstanceMessage::ShowRequested);
                    }
                    _ => {
                        // Unknown command, ignore
                    }
                }
            }
        });
    }
}

/// Clean up the socket file on shutdown.
pub fn cleanup() {
    #[cfg(not(target_os = "windows"))]
    {
        let socket_path = get_socket_path();
        let _ = std::fs::remove_file(&socket_path);
    }
}
