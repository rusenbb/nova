use nova_core::Config;
use nova_platform::Platform;

fn main() {
    let args: Vec<String> = std::env::args().collect();

    // Handle CLI flags
    if args.iter().any(|a| a == "--help" || a == "-h") {
        println!("Nova - Keyboard-driven productivity launcher");
        println!();
        println!("Usage: nova [OPTIONS]");
        println!();
        println!("Options:");
        println!("  --settings    Open settings directly");
        println!("  --help, -h    Show this help message");
        std::process::exit(0);
    }

    // Try to toggle an existing instance via IPC
    match nova_ui::try_send_toggle() {
        Ok(true) => {
            // Successfully sent toggle to existing instance
            std::process::exit(0);
        }
        Ok(false) => {
            // No existing instance, continue startup
        }
        Err(e) => {
            eprintln!("[Nova] IPC error: {}", e);
            // Continue startup anyway
        }
    }

    // Load configuration
    let config = Config::load();

    // Initialize platform
    let platform = Platform::current();

    // Discover apps
    let apps = platform.apps.discover_apps();
    println!("[Nova] Discovered {} applications", apps.len());

    // Launch the Iced UI
    if let Err(e) = nova_ui::run(config, platform, apps) {
        eprintln!("[Nova] Application error: {}", e);
        std::process::exit(1);
    }
}
