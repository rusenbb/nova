//! Install command for `nova install`.
//!
//! Installs extensions from the registry, GitHub, URLs, or local paths.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use console::style;
use flate2::read::GzDecoder;
use tar::Archive;

use super::registry;
use crate::extensions::ExtensionManifest;

/// Source types for extension installation.
enum InstallSource {
    /// Registry: publisher/name or just name
    Registry {
        name: String,
        version: Option<String>,
    },
    /// GitHub shorthand: github:user/repo
    GitHub { user: String, repo: String },
    /// Full URL (git clone)
    Url(String),
    /// Local filesystem path
    LocalPath(PathBuf),
}

/// Install extension from source.
pub fn run_install(source: &str, version: Option<&str>) -> Result<()> {
    // Parse the source
    let install_source = parse_source(source, version)?;

    // Handle registry source specially
    if let InstallSource::Registry { name, version } = &install_source {
        return install_from_registry(name, version.as_deref());
    }

    // Get or create working directory
    let work_dir = match &install_source {
        InstallSource::Registry { .. } => {
            // Already handled above with early return
            unreachable!("Registry source should be handled earlier")
        }
        InstallSource::LocalPath(path) => {
            // Use local path directly
            path.canonicalize()
                .context(format!("Path not found: {}", path.display()))?
        }
        InstallSource::GitHub { user, repo } => {
            // Clone from GitHub
            let url = format!("https://github.com/{}/{}.git", user, repo);
            println!(
                "{} Cloning {}...",
                style("→").cyan(),
                style(format!("github:{}/{}", user, repo)).bold()
            );
            clone_repo(&url)?
        }
        InstallSource::Url(url) => {
            // Clone from URL
            println!("{} Cloning {}...", style("→").cyan(), style(url).bold());
            clone_repo(url)?
        }
    };

    // Validate extension
    if !work_dir.join("nova.toml").exists() {
        bail!(
            "Not a valid extension: {} (missing nova.toml)",
            work_dir.display()
        );
    }

    // Load and validate manifest
    let manifest = ExtensionManifest::load(&work_dir)
        .map_err(|e| anyhow::anyhow!("Failed to load nova.toml: {}", e))?;

    ExtensionManifest::validate(&manifest)
        .map_err(|e| anyhow::anyhow!("Invalid manifest: {}", e))?;

    println!(
        "{} {}",
        style("✓").green().bold(),
        style("Loaded manifest from nova.toml").cyan()
    );

    // Build if needed
    let dist_index = work_dir.join("dist/index.js");
    if !dist_index.exists() {
        println!("{} Building extension...", style("→").cyan());
        build_extension(&work_dir)?;
        println!(
            "{} {}",
            style("✓").green().bold(),
            style("Built extension").cyan()
        );
    }

    // Get extensions directory
    let extensions_dir = get_extensions_dir()?;
    let dest_dir = extensions_dir.join(&manifest.extension.name);

    // Check if already installed
    if dest_dir.exists() {
        println!(
            "{} Extension '{}' is already installed. Reinstalling...",
            style("!").yellow().bold(),
            manifest.extension.name
        );
        fs::remove_dir_all(&dest_dir).context("Failed to remove existing installation")?;
    }

    // Create destination directory
    fs::create_dir_all(&dest_dir).context("Failed to create extension directory")?;

    // Copy files
    install_files(&work_dir, &dest_dir)?;

    println!(
        "{} {} {}",
        style("✓").green().bold(),
        style("Installed to").cyan(),
        style(dest_dir.display()).dim()
    );

    // Print summary
    println!();
    println!(
        "{} {}",
        style("Extension:").bold(),
        manifest.extension.title
    );
    println!(
        "{} {}",
        style("Version:").bold(),
        manifest.extension.version
    );

    if !manifest.commands.is_empty() {
        let cmd_names: Vec<_> = manifest.commands.iter().map(|c| c.name.as_str()).collect();
        println!("{} {}", style("Commands:").bold(), cmd_names.join(", "));
    }

    println!();
    println!("{}", style("Ready to use!").green().bold());

    // Cleanup temp directory if we cloned
    if matches!(
        install_source,
        InstallSource::GitHub { .. } | InstallSource::Url(_)
    ) {
        // work_dir is in temp, it will be cleaned up automatically
        // But we can explicitly remove it
        let _ = fs::remove_dir_all(&work_dir);
    }

    Ok(())
}

/// Parse source string into InstallSource.
fn parse_source(source: &str, version: Option<&str>) -> Result<InstallSource> {
    if let Some(rest) = source.strip_prefix("github:") {
        // GitHub shorthand: github:user/repo
        let parts: Vec<&str> = rest.split('/').collect();
        if parts.len() != 2 {
            bail!("Invalid GitHub shorthand. Use: github:user/repo");
        }
        Ok(InstallSource::GitHub {
            user: parts[0].to_string(),
            repo: parts[1].to_string(),
        })
    } else if source.starts_with("http://") || source.starts_with("https://") {
        // URL
        Ok(InstallSource::Url(source.to_string()))
    } else {
        // Check if it's a local path
        let path = PathBuf::from(source);
        if path.exists() {
            return Ok(InstallSource::LocalPath(path));
        }

        // Otherwise, treat as registry name (publisher/name or just name)
        Ok(InstallSource::Registry {
            name: source.to_string(),
            version: version.map(String::from),
        })
    }
}

/// Clone a git repository to a temporary directory.
fn clone_repo(url: &str) -> Result<PathBuf> {
    let temp_dir = std::env::temp_dir().join(format!("nova-install-{}", std::process::id()));

    // Remove if exists
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }

    let output = Command::new("git")
        .args(["clone", "--depth", "1", url])
        .arg(&temp_dir)
        .output()
        .context("Failed to run git clone. Is git installed?")?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        bail!("Failed to clone repository:\n{}", stderr.trim());
    }

    println!(
        "{} {}",
        style("✓").green().bold(),
        style("Cloned repository").cyan()
    );

    Ok(temp_dir)
}

/// Build extension if needed.
fn build_extension(ext_dir: &Path) -> Result<()> {
    let package_json = ext_dir.join("package.json");

    if package_json.exists() {
        // Check if node_modules exists
        let node_modules = ext_dir.join("node_modules");
        if !node_modules.exists() {
            let output = Command::new("npm")
                .arg("install")
                .current_dir(ext_dir)
                .output()
                .context("Failed to run npm install")?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("npm install failed:\n{}", stderr.trim());
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
            let error_msg = if !stderr.is_empty() {
                stderr.to_string()
            } else {
                stdout.to_string()
            };
            bail!("Build failed:\n{}", error_msg.trim());
        }
    } else {
        // No package.json - try direct esbuild
        let src_index = ext_dir.join("src/index.tsx");
        if !src_index.exists() {
            bail!("No build configuration found (missing package.json or src/index.tsx)");
        }

        fs::create_dir_all(ext_dir.join("dist"))?;

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
            bail!("esbuild failed:\n{}", stderr.trim());
        }
    }

    Ok(())
}

/// Get the extensions directory (platform-specific).
fn get_extensions_dir() -> Result<PathBuf> {
    let dir = dirs::data_dir()
        .map(|d| d.join("nova").join("extensions"))
        .unwrap_or_else(|| {
            dirs::home_dir()
                .map(|h| h.join(".nova").join("extensions"))
                .unwrap_or_else(|| PathBuf::from(".nova/extensions"))
        });

    fs::create_dir_all(&dir).context("Failed to create extensions directory")?;
    Ok(dir)
}

/// Install extension files to destination.
fn install_files(src: &Path, dest: &Path) -> Result<()> {
    // Copy nova.toml
    fs::copy(src.join("nova.toml"), dest.join("nova.toml")).context("Failed to copy nova.toml")?;

    // Copy dist/index.js
    let dist_src = src.join("dist/index.js");
    if dist_src.exists() {
        fs::create_dir_all(dest.join("dist"))?;
        fs::copy(&dist_src, dest.join("dist/index.js")).context("Failed to copy dist/index.js")?;
    }

    // Copy assets if they exist
    let assets_src = src.join("assets");
    if assets_src.exists() && assets_src.is_dir() {
        copy_dir_recursive(&assets_src, &dest.join("assets"))?;
    }

    Ok(())
}

/// Recursively copy a directory.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<()> {
    fs::create_dir_all(dst)?;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
        }
    }

    Ok(())
}

/// Install extension from registry.
fn install_from_registry(name: &str, version: Option<&str>) -> Result<()> {
    // Parse publisher/name format
    let (publisher, ext_name) = if name.contains('/') {
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() != 2 {
            bail!("Invalid extension name. Use: publisher/name");
        }
        (parts[0], parts[1])
    } else {
        // Search for the extension to find publisher
        println!(
            "{} Searching for extension '{}'...",
            style("→").cyan(),
            style(name).bold()
        );

        let rt = tokio::runtime::Runtime::new()?;
        let results = rt.block_on(registry::search(name, 1))?;

        if results.is_empty() {
            bail!("Extension '{}' not found in registry", name);
        }

        let ext = &results[0];
        println!(
            "{} Found {}/{}",
            style("✓").green(),
            style(&ext.publisher).cyan(),
            style(&ext.name).green()
        );

        // Return owned strings to satisfy borrow checker
        return install_from_registry(&format!("{}/{}", ext.publisher, ext.name), version);
    };

    println!(
        "{} Installing {}/{}{}...",
        style("→").cyan(),
        style(publisher).cyan(),
        style(ext_name).green(),
        version.map(|v| format!("@{}", v)).unwrap_or_default()
    );

    // Download from registry
    let rt = tokio::runtime::Runtime::new()?;
    let data = rt.block_on(registry::download(publisher, ext_name, version))?;

    println!("{} Downloaded {} bytes", style("✓").green(), data.len());

    // Install from tarball
    let full_name = format!("{}/{}", publisher, ext_name);
    install_from_tarball(&data, &full_name)?;

    Ok(())
}

/// Install extension from tarball data.
pub fn install_from_tarball(data: &[u8], _name: &str) -> Result<()> {
    // Create temp directory
    let temp_dir = std::env::temp_dir().join(format!("nova-install-{}", std::process::id()));
    if temp_dir.exists() {
        fs::remove_dir_all(&temp_dir)?;
    }
    fs::create_dir_all(&temp_dir)?;

    // Extract tarball
    let decoder = GzDecoder::new(data);
    let mut archive = Archive::new(decoder);
    archive
        .unpack(&temp_dir)
        .context("Failed to extract package")?;

    // Find nova.toml
    let manifest_path = temp_dir.join("nova.toml");
    if !manifest_path.exists() {
        bail!("Invalid package: missing nova.toml");
    }

    // Load manifest
    let manifest = ExtensionManifest::load(&temp_dir)
        .map_err(|e| anyhow::anyhow!("Failed to load manifest: {}", e))?;

    // Get extensions directory
    let extensions_dir = get_extensions_dir()?;
    let dest_dir = extensions_dir.join(&manifest.extension.name);

    // Check if already installed
    if dest_dir.exists() {
        println!(
            "{} Extension '{}' already installed. Reinstalling...",
            style("!").yellow(),
            manifest.extension.name
        );
        fs::remove_dir_all(&dest_dir)?;
    }

    // Create destination
    fs::create_dir_all(&dest_dir)?;

    // Copy files
    install_files(&temp_dir, &dest_dir)?;

    // Cleanup
    let _ = fs::remove_dir_all(&temp_dir);

    println!(
        "{} Installed {} v{}",
        style("✓").green().bold(),
        style(&manifest.extension.title).cyan(),
        style(&manifest.extension.version).yellow()
    );

    Ok(())
}
