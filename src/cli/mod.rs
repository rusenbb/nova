//! CLI commands for Nova.
//!
//! Provides developer tooling: create, dev, build, install, publish.

pub mod build;
pub mod create;
pub mod dev;
pub mod install;
pub mod registry;

use clap::{Parser, Subcommand};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "nova")]
#[command(about = "Keyboard-driven productivity launcher", long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Commands>,

    /// Open settings window
    #[arg(long)]
    pub settings: bool,

    /// Set global keyboard shortcut (e.g., '<Alt>space')
    #[arg(long, value_name = "KEY")]
    pub set_shortcut: Option<String>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Create a new extension or project
    Create {
        #[command(subcommand)]
        what: CreateCommands,
    },

    /// Run extension in development mode with hot reload
    Dev {
        /// Path to extension directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
    },

    /// Build extension for distribution
    Build {
        /// Path to extension directory (default: current directory)
        #[arg(default_value = ".")]
        path: String,
    },

    /// Install extension from various sources
    Install {
        /// Source: registry name, github:user/repo, URL, or local path
        source: String,

        /// Specific version to install (for registry)
        #[arg(long)]
        version: Option<String>,
    },

    /// Search extensions in the registry
    Search {
        /// Search query
        query: String,

        /// Maximum results to show
        #[arg(short, long, default_value = "10")]
        limit: usize,
    },

    /// Update installed extensions
    Update {
        /// Extension name to update (omit for all)
        name: Option<String>,
    },

    /// Publish extension to the registry
    Publish {
        /// Path to extension directory
        #[arg(default_value = ".")]
        path: PathBuf,
    },

    /// Log in to the extension registry
    Login,

    /// List installed extensions
    List,
}

#[derive(Subcommand)]
pub enum CreateCommands {
    /// Create a new Nova extension
    Extension {
        /// Extension name (will create directory with this name)
        name: String,

        /// Extension title (defaults to name in title case)
        #[arg(long)]
        title: Option<String>,

        /// Extension description
        #[arg(long)]
        description: Option<String>,

        /// Extension author
        #[arg(long)]
        author: Option<String>,
    },
}

/// Run the CLI and return whether it handled the command.
/// Returns Ok(true) if CLI handled the command, Ok(false) if no CLI command was given.
pub fn run() -> anyhow::Result<bool> {
    let cli = Cli::parse();

    // Handle legacy flags first
    if cli.settings {
        return Ok(false); // Let main.rs handle --settings
    }

    if cli.set_shortcut.is_some() {
        return Ok(false); // Let main.rs handle --set-shortcut
    }

    // Handle subcommands
    match cli.command {
        Some(Commands::Create { what }) => {
            match what {
                CreateCommands::Extension {
                    name,
                    title,
                    description,
                    author,
                } => {
                    create::create_extension(&name, title, description, author)?;
                }
            }
            Ok(true)
        }
        Some(Commands::Dev { path }) => {
            dev::run_dev(&path)?;
            Ok(true)
        }
        Some(Commands::Build { path }) => {
            build::run_build(&path)?;
            Ok(true)
        }
        Some(Commands::Install { source, version }) => {
            install::run_install(&source, version.as_deref())?;
            Ok(true)
        }
        Some(Commands::Search { query, limit }) => {
            run_search(&query, limit)?;
            Ok(true)
        }
        Some(Commands::Update { name }) => {
            run_update(name.as_deref())?;
            Ok(true)
        }
        Some(Commands::Publish { path }) => {
            run_publish(&path)?;
            Ok(true)
        }
        Some(Commands::Login) => {
            run_login()?;
            Ok(true)
        }
        Some(Commands::List) => {
            run_list()?;
            Ok(true)
        }
        None => Ok(false), // No subcommand, let main.rs handle normal launch
    }
}

/// Run search command.
fn run_search(query: &str, limit: usize) -> anyhow::Result<()> {
    let rt = tokio::runtime::Runtime::new()?;
    let extensions = rt.block_on(registry::search(query, limit))?;

    if extensions.is_empty() {
        println!("No extensions found for '{}'", query);
        return Ok(());
    }

    println!("Found {} extension(s):\n", extensions.len());
    for ext in extensions {
        println!(
            "  {}/{} - {}",
            console::style(&ext.publisher).cyan(),
            console::style(&ext.name).green(),
            ext.title
        );
        println!("    {}", ext.description);
        if let Some(v) = &ext.latest_version {
            println!(
                "    Version: {} | Downloads: {}",
                console::style(v).yellow(),
                ext.downloads
            );
        }
        println!();
    }

    Ok(())
}

/// Run update command.
fn run_update(name: Option<&str>) -> anyhow::Result<()> {
    let installed = get_installed_extensions()?;

    if installed.is_empty() {
        println!("No extensions installed.");
        return Ok(());
    }

    let rt = tokio::runtime::Runtime::new()?;
    let updates = rt.block_on(registry::check_updates(&installed))?;

    if updates.is_empty() {
        println!("All extensions are up to date.");
        return Ok(());
    }

    for update in &updates {
        if name.is_some() && name != Some(update.name.as_str()) {
            continue;
        }

        println!(
            "Updating {} {} → {}",
            console::style(&update.name).green(),
            console::style(&update.current).yellow(),
            console::style(&update.latest).cyan()
        );

        // Parse publisher/name
        let parts: Vec<&str> = update.name.split('/').collect();
        if parts.len() != 2 {
            eprintln!("  Invalid extension name: {}", update.name);
            continue;
        }

        let (publisher, ext_name) = (parts[0], parts[1]);

        // Download and install
        match rt.block_on(registry::download(
            publisher,
            ext_name,
            Some(&update.latest),
        )) {
            Ok(data) => {
                if let Err(e) = install::install_from_tarball(&data, &update.name) {
                    eprintln!("  Failed to install: {}", e);
                } else {
                    println!("  Updated successfully!");
                    if let Some(changelog) = &update.changelog {
                        println!("  Changelog: {}", changelog);
                    }
                }
            }
            Err(e) => {
                eprintln!("  Failed to download: {}", e);
            }
        }
    }

    Ok(())
}

/// Run publish command.
fn run_publish(path: &std::path::Path) -> anyhow::Result<()> {
    let token = registry::get_auth_token()?;

    println!("Publishing extension from {}...", path.display());

    let rt = tokio::runtime::Runtime::new()?;
    let result = rt.block_on(registry::publish(path, &token))?;

    println!(
        "\n{} Published {}/{} v{}",
        console::style("✓").green(),
        console::style(&result.publisher).cyan(),
        console::style(&result.name).green(),
        console::style(&result.version).yellow()
    );
    println!("  Download: {}", result.download_url);

    Ok(())
}

/// Run login command.
fn run_login() -> anyhow::Result<()> {
    println!("Opening GitHub to authenticate...\n");
    println!("Visit: {}/auth/github\n", registry::registry_url());
    println!("After authenticating, paste your token below:");

    let token: String = dialoguer::Input::new()
        .with_prompt("Token")
        .interact_text()?;

    registry::save_auth_token(&token)?;

    println!("\n{} Logged in successfully!", console::style("✓").green());
    Ok(())
}

/// Run list command.
fn run_list() -> anyhow::Result<()> {
    let installed = get_installed_extensions()?;

    if installed.is_empty() {
        println!("No extensions installed.");
        return Ok(());
    }

    println!("Installed extensions:\n");
    for (name, version) in &installed {
        println!(
            "  {} @ {}",
            console::style(name).green(),
            console::style(version).yellow()
        );
    }

    Ok(())
}

/// Get list of installed extensions with their versions.
fn get_installed_extensions() -> anyhow::Result<Vec<(String, String)>> {
    use crate::services::extensions::get_extensions_dir;

    let extensions_dir = get_extensions_dir();
    if !extensions_dir.exists() {
        return Ok(Vec::new());
    }

    let mut installed = Vec::new();

    for entry in std::fs::read_dir(&extensions_dir)? {
        let entry = entry?;
        let path = entry.path();

        if !path.is_dir() {
            continue;
        }

        let manifest_path = path.join("nova.toml");
        if !manifest_path.exists() {
            continue;
        }

        // Parse manifest to get name and version
        let content = std::fs::read_to_string(&manifest_path)?;
        let manifest: toml::Value = toml::from_str(&content)?;

        let default_name = entry.file_name().to_str().unwrap_or("unknown").to_string();

        let name = manifest
            .get("extension")
            .and_then(|e| e.get("name"))
            .and_then(|n| n.as_str())
            .map(|s| s.to_string())
            .unwrap_or(default_name);

        let version = manifest
            .get("extension")
            .and_then(|e| e.get("version"))
            .and_then(|v| v.as_str())
            .unwrap_or("0.0.0");

        // Try to get publisher from manifest or directory structure
        let publisher = manifest
            .get("extension")
            .and_then(|e| e.get("author"))
            .and_then(|a| a.as_str())
            .unwrap_or("local");

        installed.push((format!("{}/{}", publisher, name), version.to_string()));
    }

    Ok(installed)
}
