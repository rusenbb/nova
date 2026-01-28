//! Nova CLI - Developer tools (create, dev, build, install)
//!
//! This example provides the CLI without requiring GTK.
//! Useful for macOS/Windows where GTK isn't the native frontend.
//!
//! Usage:
//!   cargo run --example nova_cli -- create extension my-extension
//!   cargo run --example nova_cli -- --help

fn main() {
    match nova::cli::run() {
        Ok(true) => {} // CLI handled the command
        Ok(false) => {
            // No CLI command - show help
            eprintln!("No command specified. Run with --help for usage.");
            std::process::exit(1);
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
