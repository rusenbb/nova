//! GTK UI module for Nova launcher.
//!
//! This module contains all GTK-specific code for the Linux desktop.

mod exec;
mod ipc;
mod shortcut;
mod state;
mod window;

pub use exec::{
    copy_to_clipboard, execute_extension_command, execute_script, open_url, show_notification,
};
pub use ipc::{get_socket_path, try_send_toggle};
pub use shortcut::{ensure_shortcut_configured, print_help, set_shortcut};
pub use state::{result_to_action, CommandModeState, UIState, UIStateHandle};
pub use window::{build_ui, position_window, render_results_list};
