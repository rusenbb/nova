//! CLI commands for Nova.
//!
//! Provides developer tooling: create, dev, build, install.

pub mod build;
pub mod create;
pub mod dev;
pub mod install;

use clap::{Parser, Subcommand};

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
        /// Source: github:user/repo, URL, or local path
        source: String,
    },
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
        Some(Commands::Install { source }) => {
            install::run_install(&source)?;
            Ok(true)
        }
        None => Ok(false), // No subcommand, let main.rs handle normal launch
    }
}
