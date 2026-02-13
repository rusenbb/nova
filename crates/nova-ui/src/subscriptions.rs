use std::sync::Arc;

use iced::Subscription;

use nova_platform::Platform;

use crate::app::Message;

/// IPC listener subscription for single-instance toggle
pub fn ipc_listener() -> Subscription<Message> {
    Subscription::run_with_id("ipc_listener", ipc_stream())
}

fn ipc_stream() -> impl futures_lite::Stream<Item = Message> {
    futures_lite::stream::unfold((), |()| async {
        // Try to set up a Unix socket listener
        let socket_path = crate::ipc_socket_path();

        // Remove old socket if it exists
        let _ = std::fs::remove_file(&socket_path);

        match std::os::unix::net::UnixListener::bind(&socket_path) {
            Ok(listener) => {
                // Set non-blocking so we can yield
                listener
                    .set_nonblocking(true)
                    .expect("set_nonblocking failed");

                loop {
                    match listener.accept() {
                        Ok((mut stream, _)) => {
                            use std::io::Read;
                            let mut buf = [0u8; 256];
                            if let Ok(n) = stream.read(&mut buf) {
                                let msg = String::from_utf8_lossy(&buf[..n]).to_string();
                                return Some((Message::IpcReceived(msg), ()));
                            }
                        }
                        Err(ref e) if e.kind() == std::io::ErrorKind::WouldBlock => {
                            tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                        }
                        Err(e) => {
                            eprintln!("[Nova] IPC error: {}", e);
                            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                        }
                    }
                }
            }
            Err(e) => {
                eprintln!("[Nova] Failed to bind IPC socket: {}", e);
                // Don't retry immediately, wait a bit
                tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                Some((Message::Noop, ()))
            }
        }
    })
}

/// Clipboard polling subscription
pub fn clipboard_poll(platform: Arc<Platform>) -> Subscription<Message> {
    Subscription::run_with_id("clipboard_poll", clipboard_stream(platform))
}

fn clipboard_stream(platform: Arc<Platform>) -> impl futures_lite::Stream<Item = Message> {
    futures_lite::stream::unfold(
        (platform, String::new()),
        |(platform, last_content)| async move {
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;

            match platform.clipboard.get_text() {
                Ok(content) => {
                    if !content.trim().is_empty() && content != last_content {
                        let new_content = content.clone();
                        Some((
                            Message::ClipboardChanged(content),
                            (platform, new_content),
                        ))
                    } else {
                        Some((Message::Noop, (platform, last_content)))
                    }
                }
                Err(_) => Some((Message::Noop, (platform, last_content))),
            }
        },
    )
}
