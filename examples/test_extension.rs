//! Test extension execution
//!
//! Run with: cargo run --example test_extension

use std::path::PathBuf;

use nova::extensions::{ExtensionHost, ExtensionHostConfig};
use nova::platform;

fn main() {
    println!("=== Testing Extension Execution ===\n");

    // Get platform
    let platform = platform::current();

    // Get extensions directory
    let extensions_dir = dirs::data_dir()
        .map(|d| d.join("nova").join("extensions"))
        .unwrap_or_else(|| PathBuf::from("~/.nova/extensions"));

    println!("Extensions directory: {:?}", extensions_dir);
    println!("Directory exists: {}", extensions_dir.exists());

    // Create extension host
    let config = ExtensionHostConfig {
        extensions_dir: extensions_dir.clone(),
        platform: Some(platform),
        ..Default::default()
    };

    let mut host = match ExtensionHost::new(config) {
        Ok(h) => h,
        Err(e) => {
            eprintln!("Failed to create extension host: {}", e);
            return;
        }
    };

    println!("\nExtension count: {}", host.extension_count());
    println!("Command count: {}", host.command_count());

    // List manifests
    println!("\nLoaded extensions:");
    for (id, manifest) in host.manifests() {
        println!("  - {} ({})", manifest.extension.title, id);
        for cmd in &manifest.commands {
            println!("    * {} ({})", cmd.title, cmd.name);
        }
    }

    // Search for commands
    println!("\nSearching for 'notes'...");
    let results = host.search_commands("notes");
    for result in &results {
        println!(
            "  Found: {} ({}) [score: {}]",
            result.title, result.command_id, result.score
        );
    }

    // Execute the search command
    println!("\n=== Executing quick-notes:search ===\n");
    match host.execute_command(&"quick-notes".to_string(), "search", None) {
        Ok(result) => {
            println!("\n=== Result ===");
            // Parse and pretty print
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                println!("{}", serde_json::to_string_pretty(&parsed).unwrap());
            } else {
                println!("{}", result);
            }
        }
        Err(e) => {
            eprintln!("Execution failed: {:?}", e);
        }
    }

    // Execute the create command
    println!("\n=== Executing quick-notes:create ===\n");
    match host.execute_command(&"quick-notes".to_string(), "create", None) {
        Ok(result) => {
            println!("\n=== Result ===");
            // Parse and pretty print
            if let Ok(parsed) = serde_json::from_str::<serde_json::Value>(&result) {
                println!("{}", serde_json::to_string_pretty(&parsed).unwrap());
            } else {
                println!("{}", result);
            }
        }
        Err(e) => {
            eprintln!("Execution failed: {:?}", e);
        }
    }

    println!("\n=== Test Complete ===");
}
