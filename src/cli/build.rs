//! Build command for `nova build`.
//!
//! Bundles an extension for distribution.

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{bail, Context, Result};
use console::style;

use crate::extensions::ExtensionManifest;

/// Build extension for distribution.
pub fn run_build(path: &str) -> Result<()> {
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
    let manifest = ExtensionManifest::load(&ext_dir)
        .map_err(|e| anyhow::anyhow!("Failed to load nova.toml: {}", e))?;

    ExtensionManifest::validate(&manifest)
        .map_err(|e| anyhow::anyhow!("Invalid manifest: {}", e))?;

    println!(
        "{} {}",
        style("✓").green().bold(),
        style("Loaded manifest from nova.toml").cyan()
    );

    // Clean and create dist directory
    let dist_dir = ext_dir.join("dist");
    if dist_dir.exists() {
        fs::remove_dir_all(&dist_dir).context("Failed to clean dist directory")?;
    }
    fs::create_dir_all(&dist_dir).context("Failed to create dist directory")?;

    // Run build (npm run build or direct esbuild)
    build_typescript(&ext_dir)?;

    println!(
        "{} {}",
        style("✓").green().bold(),
        style("TypeScript compilation successful").cyan()
    );

    // Verify extension.js was created
    let extension_js = dist_dir.join("index.js");
    if !extension_js.exists() {
        bail!("Build failed: dist/index.js was not created");
    }

    let js_size = fs::metadata(&extension_js)?.len();
    println!(
        "{} {} {}",
        style("✓").green().bold(),
        style("Bundled to dist/index.js").cyan(),
        style(format_size(js_size)).dim()
    );

    // Copy nova.toml to dist
    fs::copy(ext_dir.join("nova.toml"), dist_dir.join("nova.toml"))
        .context("Failed to copy nova.toml")?;

    println!(
        "{} {}",
        style("✓").green().bold(),
        style("Copied nova.toml").cyan()
    );

    // Copy assets if they exist
    let assets_src = ext_dir.join("assets");
    let assets_dst = dist_dir.join("assets");
    let assets_count = if assets_src.exists() && assets_src.is_dir() {
        let count = copy_dir_recursive(&assets_src, &assets_dst)?;
        println!(
            "{} {} {}",
            style("✓").green().bold(),
            style("Copied assets").cyan(),
            style(format!("({} files)", count)).dim()
        );
        count
    } else {
        0
    };

    // Print summary
    println!();
    println!("{}", style("Output: dist/").bold());
    println!("  ├── nova.toml");
    println!(
        "  ├── index.js {}",
        style(format!("({})", format_size(js_size))).dim()
    );
    if assets_count > 0 {
        println!(
            "  └── assets/ {}",
            style(format!("({} files)", assets_count)).dim()
        );
    }
    println!();
    println!("{}", style("Ready for distribution!").green().bold());

    Ok(())
}

/// Run TypeScript/esbuild build.
fn build_typescript(ext_dir: &Path) -> Result<()> {
    let package_json = ext_dir.join("package.json");

    if package_json.exists() {
        // Check if node_modules exists
        let node_modules = ext_dir.join("node_modules");
        if !node_modules.exists() {
            print!("{} Installing dependencies... ", style("→").cyan());
            let output = Command::new("npm")
                .arg("install")
                .current_dir(ext_dir)
                .output()
                .context("Failed to run npm install")?;

            if !output.status.success() {
                println!("{}", style("failed").red());
                let stderr = String::from_utf8_lossy(&output.stderr);
                bail!("npm install failed: {}", stderr);
            }
            println!("{}", style("done").green());
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

        Ok(())
    } else {
        // No package.json - try direct esbuild
        let src_index = ext_dir.join("src/index.tsx");
        if !src_index.exists() {
            bail!("No build configuration found (missing package.json or src/index.tsx)");
        }

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
    }
}

/// Recursively copy a directory. Returns the number of files copied.
fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<usize> {
    fs::create_dir_all(dst)?;
    let mut count = 0;

    for entry in fs::read_dir(src)? {
        let entry = entry?;
        let src_path = entry.path();
        let dst_path = dst.join(entry.file_name());

        if src_path.is_dir() {
            count += copy_dir_recursive(&src_path, &dst_path)?;
        } else {
            fs::copy(&src_path, &dst_path)?;
            count += 1;
        }
    }

    Ok(count)
}

/// Format file size in human-readable format.
fn format_size(bytes: u64) -> String {
    if bytes < 1024 {
        format!("{}B", bytes)
    } else if bytes < 1024 * 1024 {
        format!("{:.1}KB", bytes as f64 / 1024.0)
    } else {
        format!("{:.1}MB", bytes as f64 / (1024.0 * 1024.0))
    }
}
