//! Development server for `nova dev`.
//!
//! Watches extension source files and hot-reloads on changes.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::channel;
use std::sync::Arc;
use std::time::Duration;

use anyhow::{bail, Context, Result};
use console::style;
use notify_debouncer_mini::{new_debouncer, notify::RecursiveMode, DebouncedEventKind};

use crate::extensions::ExtensionManifest;

/// Run extension in development mode with hot reload.
pub fn run_dev(path: &str) -> Result<()> {
    let ext_dir = PathBuf::from(path)
        .canonicalize()
        .context(format!("Extension directory not found: {}", path))?;

    // Validate extension directory
    if !ext_dir.join("nova.toml").exists() {
        bail!(
            "Not an extension directory: {} (missing nova.toml)",
            ext_dir.display()
        );
    }

    // Load and validate manifest
    let manifest = ExtensionManifest::load(&ext_dir).map_err(|e| {
        anyhow::anyhow!("Failed to load nova.toml from {}: {}", ext_dir.display(), e)
    })?;

    manifest.validate().context("Invalid manifest")?;

    println!(
        "{} {} {}",
        style("✓").green().bold(),
        style("Loaded manifest:").cyan(),
        style(&manifest.extension.title).bold()
    );

    // Initial build
    if let Err(e) = run_build(&ext_dir) {
        eprintln!("{} Build failed: {}", style("✗").red().bold(), e);
        eprintln!("  Watching for changes...");
    } else {
        println!(
            "{} {}",
            style("✓").green().bold(),
            style("Build successful").cyan()
        );
    }

    // Verify dist/index.js exists
    let dist_path = ext_dir.join("dist/index.js");
    if dist_path.exists() {
        println!(
            "{} {} {}",
            style("✓").green().bold(),
            style("Extension ready:").cyan(),
            style(&manifest.extension.name).bold()
        );
    }

    println!();
    println!("{}", style("Watching for changes...").dim());
    println!("{}", style("Press Ctrl+C to stop.").dim());
    println!();

    // Set up Ctrl+C handler
    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, Ordering::SeqCst);
    })?;

    // Set up file watcher
    let (tx, rx) = channel();
    let mut debouncer = new_debouncer(Duration::from_millis(300), tx)?;

    // Watch src directory
    let src_dir = ext_dir.join("src");
    if src_dir.exists() {
        debouncer
            .watcher()
            .watch(&src_dir, RecursiveMode::Recursive)?;
    }

    // Watch nova.toml
    debouncer
        .watcher()
        .watch(&ext_dir.join("nova.toml"), RecursiveMode::NonRecursive)?;

    // Main event loop
    while running.load(Ordering::SeqCst) {
        match rx.recv_timeout(Duration::from_millis(100)) {
            Ok(Ok(events)) => {
                // Filter for actual changes
                let has_changes = events.iter().any(|e| {
                    matches!(e.kind, DebouncedEventKind::Any)
                        && e.path.extension().map_or(false, |ext| {
                            matches!(
                                ext.to_str(),
                                Some("ts") | Some("tsx") | Some("js") | Some("toml")
                            )
                        })
                });

                if has_changes {
                    let changed_files: Vec<_> = events
                        .iter()
                        .filter_map(|e| e.path.file_name())
                        .map(|n| n.to_string_lossy().to_string())
                        .collect();

                    println!(
                        "{} File changed: {}",
                        style("[").dim(),
                        style(changed_files.join(", ")).yellow()
                    );

                    // Reload manifest if nova.toml changed
                    let manifest_changed = events
                        .iter()
                        .any(|e| e.path.file_name().map_or(false, |n| n == "nova.toml"));

                    if manifest_changed {
                        match ExtensionManifest::load(&ext_dir) {
                            Ok(new_manifest) => {
                                if let Err(e) = ExtensionManifest::validate(&new_manifest) {
                                    eprintln!(
                                        "{} Invalid manifest: {}",
                                        style("✗").red().bold(),
                                        e
                                    );
                                    continue;
                                }
                                println!("{} Manifest reloaded", style("✓").green().bold());
                            }
                            Err(e) => {
                                eprintln!(
                                    "{} Failed to reload manifest: {}",
                                    style("✗").red().bold(),
                                    e
                                );
                                continue;
                            }
                        }
                    }

                    // Rebuild
                    print!("{} Rebuilding... ", style("→").cyan());
                    match run_build(&ext_dir) {
                        Ok(()) => {
                            println!("{}", style("done").green());
                            println!(
                                "{} {}",
                                style("✓").green().bold(),
                                style("Reload complete").cyan()
                            );
                        }
                        Err(e) => {
                            println!("{}", style("failed").red());
                            eprintln!("  {}", e);
                        }
                    }
                    println!();
                }
            }
            Ok(Err(e)) => {
                eprintln!("Watch error: {:?}", e);
            }
            Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                // Normal timeout, continue
            }
            Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                break;
            }
        }
    }

    println!();
    println!("{}", style("Development server stopped.").dim());
    Ok(())
}

/// Run esbuild to compile TypeScript.
fn run_build(ext_dir: &Path) -> Result<()> {
    let package_json = ext_dir.join("package.json");

    // Check if package.json exists with build script
    if package_json.exists() {
        // Check if node_modules exists
        let node_modules = ext_dir.join("node_modules");
        if !node_modules.exists() {
            // Run npm install first
            let output = Command::new("npm")
                .arg("install")
                .current_dir(ext_dir)
                .output()
                .context("Failed to run npm install")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("npm install failed: {}", stderr);
            }
        }

        // Run npm run build
        let output = Command::new("npm")
            .args(["run", "build"])
            .current_dir(ext_dir)
            .output()
            .context("Failed to run npm run build")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);

            // esbuild errors go to stderr, TypeScript errors to stdout
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };

            bail!("{}", error_msg.trim());
        }

        Ok(())
    } else {
        // No package.json - try direct esbuild if src/index.tsx exists
        let src_index = ext_dir.join("src/index.tsx");
        if src_index.exists() {
            std::fs::create_dir_all(ext_dir.join("dist"))?;

            let output = Command::new("npx")
                .args([
                    "esbuild",
                    "src/index.tsx",
                    "--bundle",
                    "--outfile=dist/index.js",
                    "--format=esm",
                    "--external:@aspect/nova",
                ])
                .current_dir(ext_dir)
                .output()
                .context("Failed to run esbuild")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("esbuild failed: {}", stderr);
            }

            Ok(())
        } else {
            bail!("No build configuration found (missing package.json or src/index.tsx)");
        }
    }
}
