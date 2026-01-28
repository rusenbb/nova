//! Background execution scheduler for extensions.
//!
//! This module provides the infrastructure for running extension background tasks
//! at configurable intervals. Key features:
//! - Periodic execution based on manifest configuration
//! - Battery-aware throttling to reduce power consumption
//! - Per-extension user toggles for background execution
//! - Thread-safe scheduling with graceful shutdown

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::{Duration, Instant};

use tokio::sync::{mpsc, Mutex, RwLock};
use tokio::task::JoinHandle;

use super::error::{ExtensionError, ExtensionResult};
use super::manifest::BackgroundConfig;
use super::ExtensionId;

/// Power state of the device.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum PowerState {
    /// Device is connected to AC power.
    AcPower,
    /// Device is running on battery.
    Battery,
    /// Power state is unknown (assume AC).
    #[default]
    Unknown,
}

/// Configuration for the background scheduler.
#[derive(Debug, Clone)]
pub struct BackgroundSchedulerConfig {
    /// Directory for storing background settings.
    pub settings_dir: PathBuf,

    /// Minimum interval between background checks (in seconds).
    /// Extensions cannot run more frequently than this.
    pub min_interval: u64,

    /// Multiplier for intervals when on battery power.
    /// Set to 2.0 to double intervals when on battery.
    pub battery_throttle_multiplier: f64,

    /// Whether to pause non-critical background tasks on battery.
    pub pause_on_battery: bool,

    /// How often to check power state (in seconds).
    pub power_check_interval: u64,
}

impl Default for BackgroundSchedulerConfig {
    fn default() -> Self {
        Self {
            settings_dir: dirs::data_dir()
                .map(|d| d.join("nova").join("background"))
                .unwrap_or_else(|| PathBuf::from("~/.nova/background")),
            min_interval: 60,
            battery_throttle_multiplier: 2.0,
            pause_on_battery: false,
            power_check_interval: 60,
        }
    }
}

/// State for a single extension's background task.
#[derive(Debug, Clone)]
struct BackgroundTaskState {
    /// Extension identifier (used for logging and debugging).
    #[allow(dead_code)]
    extension_id: ExtensionId,

    /// Configuration from manifest.
    config: BackgroundConfig,

    /// Whether background is enabled by user (can override manifest).
    user_enabled: bool,

    /// Last time the background task ran.
    last_run: Option<Instant>,

    /// Number of consecutive failures.
    failure_count: u32,

    /// Whether this extension's background is critical (should run even on battery).
    is_critical: bool,
}

impl BackgroundTaskState {
    fn new(extension_id: ExtensionId, config: BackgroundConfig) -> Self {
        Self {
            extension_id,
            config,
            user_enabled: true, // Enabled by default if manifest has background
            last_run: None,
            failure_count: 0,
            is_critical: false,
        }
    }

    /// Calculate the effective interval, accounting for throttling.
    fn effective_interval(&self, power_state: PowerState, throttle_multiplier: f64) -> Duration {
        let base = Duration::from_secs(self.config.interval);

        // Apply exponential backoff for failures (max 8x)
        let failure_multiplier = 2.0_f64.powi(self.failure_count.min(3) as i32);

        // Apply battery throttle
        let power_multiplier = match power_state {
            PowerState::Battery => throttle_multiplier,
            _ => 1.0,
        };

        let total_multiplier = failure_multiplier * power_multiplier;
        Duration::from_secs_f64(base.as_secs_f64() * total_multiplier)
    }

    /// Check if this task should run now.
    fn should_run(&self, power_state: PowerState, config: &BackgroundSchedulerConfig) -> bool {
        // Check user toggle
        if !self.user_enabled {
            return false;
        }

        // Check battery pause (unless critical)
        if config.pause_on_battery && power_state == PowerState::Battery && !self.is_critical {
            return false;
        }

        // Check timing
        let effective_interval =
            self.effective_interval(power_state, config.battery_throttle_multiplier);

        match self.last_run {
            Some(last) => last.elapsed() >= effective_interval,
            None => self.config.run_on_load,
        }
    }
}

/// Message types for communicating with the scheduler.
#[derive(Debug)]
pub enum SchedulerMessage {
    /// Register an extension for background execution.
    Register {
        extension_id: ExtensionId,
        config: BackgroundConfig,
    },

    /// Unregister an extension.
    Unregister { extension_id: ExtensionId },

    /// Enable or disable background for an extension (user toggle).
    SetEnabled {
        extension_id: ExtensionId,
        enabled: bool,
    },

    /// Mark an extension as critical (runs even on battery).
    SetCritical {
        extension_id: ExtensionId,
        critical: bool,
    },

    /// Force an immediate background tick for an extension.
    ForceTick { extension_id: ExtensionId },

    /// Shutdown the scheduler.
    Shutdown,
}

/// Callback function type for executing background tasks.
/// Returns Ok(()) on success, Err with message on failure.
pub type BackgroundCallback =
    Arc<dyn Fn(&ExtensionId) -> ExtensionResult<()> + Send + Sync + 'static>;

/// Background scheduler that manages periodic execution of extension tasks.
///
/// The scheduler runs in its own tokio task and periodically checks which
/// extensions need their background handlers called.
pub struct BackgroundScheduler {
    /// Configuration (stored for potential runtime access).
    #[allow(dead_code)]
    config: BackgroundSchedulerConfig,

    /// Channel for sending messages to the scheduler task.
    tx: mpsc::Sender<SchedulerMessage>,

    /// Handle to the scheduler task.
    task_handle: Option<JoinHandle<()>>,

    /// Shared state for extensions.
    extensions: Arc<RwLock<HashMap<ExtensionId, BackgroundTaskState>>>,

    /// Current power state.
    power_state: Arc<RwLock<PowerState>>,

    /// Whether the scheduler is running.
    running: Arc<AtomicBool>,
}

impl BackgroundScheduler {
    /// Create a new background scheduler.
    ///
    /// The scheduler starts immediately and runs until shutdown is called.
    pub fn new(config: BackgroundSchedulerConfig, callback: BackgroundCallback) -> Self {
        let (tx, rx) = mpsc::channel(64);
        let extensions: Arc<RwLock<HashMap<ExtensionId, BackgroundTaskState>>> =
            Arc::new(RwLock::new(HashMap::new()));
        let power_state = Arc::new(RwLock::new(PowerState::Unknown));
        let running = Arc::new(AtomicBool::new(true));

        // Clone for the spawned task
        let extensions_clone = Arc::clone(&extensions);
        let power_state_clone = Arc::clone(&power_state);
        let running_clone = Arc::clone(&running);
        let config_clone = config.clone();

        // Spawn the scheduler task
        let task_handle = tokio::spawn(async move {
            scheduler_loop(
                rx,
                extensions_clone,
                power_state_clone,
                running_clone,
                config_clone,
                callback,
            )
            .await;
        });

        Self {
            config,
            tx,
            task_handle: Some(task_handle),
            extensions,
            power_state,
            running,
        }
    }

    /// Register an extension for background execution.
    pub async fn register(
        &self,
        extension_id: ExtensionId,
        config: BackgroundConfig,
    ) -> ExtensionResult<()> {
        self.tx
            .send(SchedulerMessage::Register {
                extension_id,
                config,
            })
            .await
            .map_err(|e| ExtensionError::ExecutionError(format!("Failed to send message: {}", e)))
    }

    /// Unregister an extension.
    pub async fn unregister(&self, extension_id: &ExtensionId) -> ExtensionResult<()> {
        self.tx
            .send(SchedulerMessage::Unregister {
                extension_id: extension_id.clone(),
            })
            .await
            .map_err(|e| ExtensionError::ExecutionError(format!("Failed to send message: {}", e)))
    }

    /// Enable or disable background for an extension (user toggle).
    pub async fn set_enabled(
        &self,
        extension_id: &ExtensionId,
        enabled: bool,
    ) -> ExtensionResult<()> {
        self.tx
            .send(SchedulerMessage::SetEnabled {
                extension_id: extension_id.clone(),
                enabled,
            })
            .await
            .map_err(|e| ExtensionError::ExecutionError(format!("Failed to send message: {}", e)))
    }

    /// Mark an extension as critical (runs even on battery).
    pub async fn set_critical(
        &self,
        extension_id: &ExtensionId,
        critical: bool,
    ) -> ExtensionResult<()> {
        self.tx
            .send(SchedulerMessage::SetCritical {
                extension_id: extension_id.clone(),
                critical,
            })
            .await
            .map_err(|e| ExtensionError::ExecutionError(format!("Failed to send message: {}", e)))
    }

    /// Force an immediate background tick for an extension.
    pub async fn force_tick(&self, extension_id: &ExtensionId) -> ExtensionResult<()> {
        self.tx
            .send(SchedulerMessage::ForceTick {
                extension_id: extension_id.clone(),
            })
            .await
            .map_err(|e| ExtensionError::ExecutionError(format!("Failed to send message: {}", e)))
    }

    /// Get the current power state.
    pub async fn power_state(&self) -> PowerState {
        *self.power_state.read().await
    }

    /// Check if background is enabled for an extension.
    pub async fn is_enabled(&self, extension_id: &ExtensionId) -> bool {
        self.extensions
            .read()
            .await
            .get(extension_id)
            .map(|s| s.user_enabled)
            .unwrap_or(false)
    }

    /// Get a list of registered extension IDs.
    pub async fn registered_extensions(&self) -> Vec<ExtensionId> {
        self.extensions.read().await.keys().cloned().collect()
    }

    /// Shutdown the scheduler.
    pub async fn shutdown(&mut self) -> ExtensionResult<()> {
        self.running.store(false, Ordering::SeqCst);

        let _ = self.tx.send(SchedulerMessage::Shutdown).await;

        if let Some(handle) = self.task_handle.take() {
            handle.await.map_err(|e| {
                ExtensionError::ExecutionError(format!("Scheduler task panicked: {}", e))
            })?;
        }

        Ok(())
    }
}

/// Main scheduler loop that runs in a tokio task.
async fn scheduler_loop(
    mut rx: mpsc::Receiver<SchedulerMessage>,
    extensions: Arc<RwLock<HashMap<ExtensionId, BackgroundTaskState>>>,
    power_state: Arc<RwLock<PowerState>>,
    running: Arc<AtomicBool>,
    config: BackgroundSchedulerConfig,
    callback: BackgroundCallback,
) {
    let tick_interval = Duration::from_secs(config.min_interval.max(10));
    let power_check_interval = Duration::from_secs(config.power_check_interval);
    let mut last_power_check = Instant::now();

    // Load persisted user settings
    load_user_settings(&config.settings_dir, &extensions).await;

    while running.load(Ordering::SeqCst) {
        // Check for messages (non-blocking with timeout)
        match tokio::time::timeout(tick_interval, rx.recv()).await {
            Ok(Some(msg)) => {
                handle_message(msg, &extensions, &config).await;
            }
            Ok(None) => {
                // Channel closed, exit
                break;
            }
            Err(_) => {
                // Timeout - proceed with tick
            }
        }

        // Check power state periodically
        if last_power_check.elapsed() >= power_check_interval {
            let new_state = detect_power_state();
            *power_state.write().await = new_state;
            last_power_check = Instant::now();
        }

        // Get current power state
        let current_power = *power_state.read().await;

        // Check each extension
        let mut to_run = Vec::new();
        {
            let exts = extensions.read().await;
            for (ext_id, state) in exts.iter() {
                if state.should_run(current_power, &config) {
                    to_run.push(ext_id.clone());
                }
            }
        }

        // Execute background tasks
        for ext_id in to_run {
            let result = callback(&ext_id);

            // Update state based on result
            let mut exts = extensions.write().await;
            if let Some(state) = exts.get_mut(&ext_id) {
                state.last_run = Some(Instant::now());
                match result {
                    Ok(()) => {
                        state.failure_count = 0;
                    }
                    Err(e) => {
                        state.failure_count = state.failure_count.saturating_add(1);
                        eprintln!(
                            "[nova] Background task failed for {}: {} (failures: {})",
                            ext_id, e, state.failure_count
                        );
                    }
                }
            }
        }
    }

    // Save user settings before exit
    save_user_settings(&config.settings_dir, &extensions).await;
}

/// Handle a scheduler message.
async fn handle_message(
    msg: SchedulerMessage,
    extensions: &Arc<RwLock<HashMap<ExtensionId, BackgroundTaskState>>>,
    config: &BackgroundSchedulerConfig,
) {
    match msg {
        SchedulerMessage::Register {
            extension_id,
            config: bg_config,
        } => {
            let state = BackgroundTaskState::new(extension_id.clone(), bg_config);
            extensions.write().await.insert(extension_id, state);
        }

        SchedulerMessage::Unregister { extension_id } => {
            extensions.write().await.remove(&extension_id);
            // Save settings after removal
            save_user_settings(&config.settings_dir, extensions).await;
        }

        SchedulerMessage::SetEnabled {
            extension_id,
            enabled,
        } => {
            if let Some(state) = extensions.write().await.get_mut(&extension_id) {
                state.user_enabled = enabled;
                // Persist the setting
                save_user_settings(&config.settings_dir, extensions).await;
            }
        }

        SchedulerMessage::SetCritical {
            extension_id,
            critical,
        } => {
            if let Some(state) = extensions.write().await.get_mut(&extension_id) {
                state.is_critical = critical;
            }
        }

        SchedulerMessage::ForceTick { extension_id } => {
            // Reset last_run to force immediate execution on next tick
            if let Some(state) = extensions.write().await.get_mut(&extension_id) {
                state.last_run = None;
            }
        }

        SchedulerMessage::Shutdown => {
            // Handled by the loop
        }
    }
}

/// Detect the current power state.
///
/// This function is platform-specific:
/// - macOS: Uses IOKit to check power source
/// - Linux: Checks /sys/class/power_supply
/// - Windows: Uses GetSystemPowerStatus (TODO)
pub fn detect_power_state() -> PowerState {
    #[cfg(target_os = "macos")]
    {
        detect_power_state_macos()
    }

    #[cfg(target_os = "linux")]
    {
        detect_power_state_linux()
    }

    #[cfg(target_os = "windows")]
    {
        // TODO: Implement Windows power detection
        PowerState::Unknown
    }

    #[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
    {
        PowerState::Unknown
    }
}

#[cfg(target_os = "macos")]
fn detect_power_state_macos() -> PowerState {
    use std::process::Command;

    // Use pmset to check power source
    // This is simpler than using IOKit directly and doesn't require additional dependencies
    match Command::new("pmset").args(["-g", "ps"]).output() {
        Ok(output) => {
            let stdout = String::from_utf8_lossy(&output.stdout);
            if stdout.contains("AC Power") {
                PowerState::AcPower
            } else if stdout.contains("Battery Power") {
                PowerState::Battery
            } else {
                PowerState::Unknown
            }
        }
        Err(_) => PowerState::Unknown,
    }
}

#[cfg(target_os = "linux")]
fn detect_power_state_linux() -> PowerState {
    use std::fs;
    use std::path::Path;

    let power_supply_dir = Path::new("/sys/class/power_supply");

    if !power_supply_dir.exists() {
        return PowerState::Unknown;
    }

    // Look for AC adapter or battery status
    if let Ok(entries) = fs::read_dir(power_supply_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            let type_path = path.join("type");

            if let Ok(power_type) = fs::read_to_string(&type_path) {
                let power_type = power_type.trim();

                // Check for AC adapter
                if power_type == "Mains" {
                    let online_path = path.join("online");
                    if let Ok(online) = fs::read_to_string(&online_path) {
                        if online.trim() == "1" {
                            return PowerState::AcPower;
                        }
                    }
                }

                // Check for battery
                if power_type == "Battery" {
                    let status_path = path.join("status");
                    if let Ok(status) = fs::read_to_string(&status_path) {
                        let status = status.trim().to_lowercase();
                        if status == "discharging" {
                            return PowerState::Battery;
                        } else if status == "charging" || status == "full" {
                            return PowerState::AcPower;
                        }
                    }
                }
            }
        }
    }

    PowerState::Unknown
}

/// User settings for background execution.
#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
struct UserSettings {
    /// Per-extension enable/disable state.
    enabled: HashMap<ExtensionId, bool>,
}

/// Load user settings from disk.
async fn load_user_settings(
    settings_dir: &Path,
    extensions: &Arc<RwLock<HashMap<ExtensionId, BackgroundTaskState>>>,
) {
    let settings_file = settings_dir.join("settings.json");

    if !settings_file.exists() {
        return;
    }

    match std::fs::read_to_string(&settings_file) {
        Ok(content) => match serde_json::from_str::<UserSettings>(&content) {
            Ok(settings) => {
                let mut exts = extensions.write().await;
                for (ext_id, enabled) in settings.enabled {
                    if let Some(state) = exts.get_mut(&ext_id) {
                        state.user_enabled = enabled;
                    }
                }
            }
            Err(e) => {
                eprintln!("[nova] Failed to parse background settings: {}", e);
            }
        },
        Err(e) => {
            eprintln!("[nova] Failed to read background settings: {}", e);
        }
    }
}

/// Save user settings to disk.
async fn save_user_settings(
    settings_dir: &Path,
    extensions: &Arc<RwLock<HashMap<ExtensionId, BackgroundTaskState>>>,
) {
    let settings_file = settings_dir.join("settings.json");

    // Collect settings
    let settings = {
        let exts = extensions.read().await;
        UserSettings {
            enabled: exts
                .iter()
                .map(|(id, state)| (id.clone(), state.user_enabled))
                .collect(),
        }
    };

    // Ensure directory exists
    if let Err(e) = std::fs::create_dir_all(settings_dir) {
        eprintln!("[nova] Failed to create background settings dir: {}", e);
        return;
    }

    // Write settings
    match serde_json::to_string_pretty(&settings) {
        Ok(content) => {
            if let Err(e) = std::fs::write(&settings_file, content) {
                eprintln!("[nova] Failed to write background settings: {}", e);
            }
        }
        Err(e) => {
            eprintln!("[nova] Failed to serialize background settings: {}", e);
        }
    }
}

/// Blocking wrapper for background scheduler operations.
///
/// This struct provides a synchronous interface for FFI calls.
pub struct BackgroundSchedulerHandle {
    scheduler: Arc<Mutex<BackgroundScheduler>>,
    runtime: tokio::runtime::Handle,
}

impl BackgroundSchedulerHandle {
    /// Create a new handle wrapping a scheduler.
    pub fn new(scheduler: BackgroundScheduler, runtime: tokio::runtime::Handle) -> Self {
        Self {
            scheduler: Arc::new(Mutex::new(scheduler)),
            runtime,
        }
    }

    /// Set enabled state for an extension (blocking).
    pub fn set_enabled_blocking(&self, extension_id: &ExtensionId, enabled: bool) -> bool {
        let scheduler = Arc::clone(&self.scheduler);
        let ext_id = extension_id.clone();

        self.runtime.block_on(async move {
            let scheduler = scheduler.lock().await;
            scheduler.set_enabled(&ext_id, enabled).await.is_ok()
        })
    }

    /// Check if background is enabled for an extension (blocking).
    pub fn is_enabled_blocking(&self, extension_id: &ExtensionId) -> bool {
        let scheduler = Arc::clone(&self.scheduler);
        let ext_id = extension_id.clone();

        self.runtime.block_on(async move {
            let scheduler = scheduler.lock().await;
            scheduler.is_enabled(&ext_id).await
        })
    }

    /// Get current power state (blocking).
    pub fn power_state_blocking(&self) -> PowerState {
        let scheduler = Arc::clone(&self.scheduler);

        self.runtime.block_on(async move {
            let scheduler = scheduler.lock().await;
            scheduler.power_state().await
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_background_config_defaults() {
        let config = BackgroundSchedulerConfig::default();
        assert_eq!(config.min_interval, 60);
        assert!((config.battery_throttle_multiplier - 2.0).abs() < f64::EPSILON);
        assert!(!config.pause_on_battery);
    }

    #[test]
    fn test_power_state_default() {
        let state = PowerState::default();
        assert_eq!(state, PowerState::Unknown);
    }

    #[test]
    fn test_task_state_effective_interval() {
        let config = BackgroundConfig {
            interval: 300,
            run_on_load: false,
        };
        let mut state = BackgroundTaskState::new("test".to_string(), config);

        // No throttle on AC
        let interval = state.effective_interval(PowerState::AcPower, 2.0);
        assert_eq!(interval, Duration::from_secs(300));

        // 2x throttle on battery
        let interval = state.effective_interval(PowerState::Battery, 2.0);
        assert_eq!(interval, Duration::from_secs(600));

        // Failure backoff
        state.failure_count = 1;
        let interval = state.effective_interval(PowerState::AcPower, 2.0);
        assert_eq!(interval, Duration::from_secs(600)); // 2x for 1 failure

        state.failure_count = 2;
        let interval = state.effective_interval(PowerState::AcPower, 2.0);
        assert_eq!(interval, Duration::from_secs(1200)); // 4x for 2 failures
    }

    #[test]
    fn test_task_state_should_run() {
        let config = BackgroundConfig {
            interval: 300,
            run_on_load: true,
        };
        let state = BackgroundTaskState::new("test".to_string(), config);
        let scheduler_config = BackgroundSchedulerConfig::default();

        // Should run immediately if run_on_load is true and never ran
        assert!(state.should_run(PowerState::AcPower, &scheduler_config));
    }

    #[test]
    fn test_task_state_user_disabled() {
        let config = BackgroundConfig {
            interval: 300,
            run_on_load: true,
        };
        let mut state = BackgroundTaskState::new("test".to_string(), config);
        state.user_enabled = false;
        let scheduler_config = BackgroundSchedulerConfig::default();

        // Should not run if user disabled
        assert!(!state.should_run(PowerState::AcPower, &scheduler_config));
    }

    #[test]
    fn test_detect_power_state() {
        // Just ensure it doesn't panic
        let _ = detect_power_state();
    }
}
