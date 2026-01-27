//! Nova GTK frontend for Linux.

mod config;
mod core;
mod error;
mod executor;
mod gtk;
mod platform;
mod search;
mod services;
mod settings;

use ::gtk::prelude::*;
use ::gtk::Application;
use std::env;

const APP_ID: &str = "com.rusen.nova";

fn main() {
    // Try CLI commands first (create, dev, build, install)
    match nova::cli::run() {
        Ok(true) => return, // CLI handled the command
        Ok(false) => {}     // No CLI command, continue to GTK
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }

    let args: Vec<String> = env::args().collect();

    // Handle legacy CLI arguments (--settings, --set-shortcut)
    if args.len() > 1 {
        match args[1].as_str() {
            "--help" | "-h" => {
                gtk::print_help();
                return;
            }
            "--settings" => {
                let app = Application::builder()
                    .application_id(format!("{}.settings", APP_ID))
                    .build();

                app.connect_activate(|app| {
                    settings::show_settings_window(app);
                });

                // Pass empty args to avoid GTK parsing our custom args
                app.run_with_args(&[] as &[&str]);
                return;
            }
            "--set-shortcut" => {
                if args.len() < 3 {
                    eprintln!("Error: --set-shortcut requires a shortcut argument");
                    eprintln!("Example: nova --set-shortcut '<Alt>space'");
                    std::process::exit(1);
                }
                match gtk::set_shortcut(&args[2]) {
                    Ok(()) => return,
                    Err(e) => {
                        eprintln!("Error: {}", e);
                        std::process::exit(1);
                    }
                }
            }
            arg if arg.starts_with('-') => {
                // Unknown flag - clap already handled valid subcommands
                eprintln!("Unknown argument: {}", args[1]);
                eprintln!("Run 'nova --help' for usage information");
                std::process::exit(1);
            }
            _ => {
                // Positional arg without subcommand - show help
                eprintln!("Unknown command: {}", args[1]);
                eprintln!("Run 'nova --help' for usage information");
                std::process::exit(1);
            }
        }
    }

    if gtk::try_send_toggle() {
        println!("[Nova] Sent toggle to existing instance");
        return;
    }

    // Ensure keyboard shortcut is configured on startup
    gtk::ensure_shortcut_configured();

    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(gtk::build_ui);
    app.run();
}
