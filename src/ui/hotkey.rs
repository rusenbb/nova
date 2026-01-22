//! Global hotkey management for Nova.
//!
//! This module provides cross-platform global hotkey registration using the
//! `global-hotkey` crate and integrates it with the iced subscription system.

use global_hotkey::{
    hotkey::{Code, HotKey, Modifiers},
    GlobalHotKeyEvent, GlobalHotKeyManager, HotKeyState,
};
use iced::futures::stream;
use iced::futures::Stream;
use once_cell::sync::OnceCell;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;

/// Global hotkey manager singleton.
static HOTKEY_MANAGER: OnceCell<HotkeyManagerHandle> = OnceCell::new();

/// Handle to the hotkey manager running in a background thread.
struct HotkeyManagerHandle {
    /// Channel to receive hotkey events.
    _sender: Sender<HotkeyMessage>,
}

/// Messages from the hotkey system.
#[derive(Debug, Clone)]
pub enum HotkeyMessage {
    /// The toggle hotkey was pressed.
    TogglePressed,
    /// Failed to register the hotkey.
    RegistrationFailed(String),
}

/// Configuration for the global hotkey.
#[derive(Debug, Clone)]
pub struct HotkeyConfig {
    /// Primary modifier (e.g., Alt, Command, Super).
    pub modifier: HotkeyModifier,
    /// The key code (e.g., Space).
    pub key: HotkeyKey,
}

impl Default for HotkeyConfig {
    fn default() -> Self {
        Self {
            modifier: HotkeyModifier::Alt,
            key: HotkeyKey::Space,
        }
    }
}

/// Supported modifiers for the hotkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyModifier {
    Alt,
    Command,
    Control,
    Shift,
    Super,
}

impl HotkeyModifier {
    fn to_modifiers(self) -> Modifiers {
        match self {
            HotkeyModifier::Alt => Modifiers::ALT,
            HotkeyModifier::Command => Modifiers::META,
            HotkeyModifier::Control => Modifiers::CONTROL,
            HotkeyModifier::Shift => Modifiers::SHIFT,
            HotkeyModifier::Super => Modifiers::SUPER,
        }
    }
}

/// Supported key codes for the hotkey.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HotkeyKey {
    Space,
    KeyN,
    KeyP,
}

impl HotkeyKey {
    fn to_code(self) -> Code {
        match self {
            HotkeyKey::Space => Code::Space,
            HotkeyKey::KeyN => Code::KeyN,
            HotkeyKey::KeyP => Code::KeyP,
        }
    }
}

/// Initialize the global hotkey system.
///
/// Returns a channel receiver for hotkey events, or None if already initialized.
pub fn init_hotkey_manager(config: &HotkeyConfig) -> Option<Receiver<HotkeyMessage>> {
    let (tx, rx) = mpsc::channel();

    let config = config.clone();
    let tx_clone = tx.clone();

    // The hotkey manager must be created and managed on a background thread
    // because it requires an event loop on some platforms.
    thread::spawn(move || {
        if let Err(e) = run_hotkey_manager(&config, tx_clone) {
            eprintln!("[Nova Hotkey] Failed to initialize: {}", e);
        }
    });

    // Store the handle (sender is kept to prevent the channel from closing)
    let _ = HOTKEY_MANAGER.set(HotkeyManagerHandle { _sender: tx });

    Some(rx)
}

/// Run the hotkey manager (called from background thread).
fn run_hotkey_manager(config: &HotkeyConfig, tx: Sender<HotkeyMessage>) -> Result<(), String> {
    let manager = GlobalHotKeyManager::new()
        .map_err(|e| format!("Failed to create hotkey manager: {}", e))?;

    // Build the hotkey from config
    let hotkey = HotKey::new(Some(config.modifier.to_modifiers()), config.key.to_code());

    manager
        .register(hotkey)
        .map_err(|e| format!("Failed to register hotkey: {}", e))?;

    println!("[Nova Hotkey] Registered global hotkey: {:?}+{:?}", config.modifier, config.key);

    // Process hotkey events
    loop {
        if let Ok(event) = GlobalHotKeyEvent::receiver().recv() {
            if event.state == HotKeyState::Pressed {
                let _ = tx.send(HotkeyMessage::TogglePressed);
            }
        }
    }
}

/// Create a stream of hotkey events from an Arc<Mutex<Receiver>>.
pub fn hotkey_stream_arc(
    rx: std::sync::Arc<std::sync::Mutex<Receiver<HotkeyMessage>>>,
) -> impl Stream<Item = HotkeyMessage> {
    stream::unfold(rx, |rx| async move {
        loop {
            let result = {
                let guard = rx.lock().ok()?;
                guard.try_recv()
            };
            match result {
                Ok(msg) => return Some((msg, rx)),
                Err(mpsc::TryRecvError::Empty) => {
                    // Small delay to avoid busy-waiting
                    tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    return None;
                }
            }
        }
    })
}
