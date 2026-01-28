//! Extension scanning and validation.

use flate2::read::GzDecoder;
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::io::Read;
use tar::Archive;

use crate::api::ApiError;

/// Maximum allowed package size (10 MB).
const MAX_PACKAGE_SIZE: usize = 10 * 1024 * 1024;

/// Result of scanning an extension package.
pub struct ScanResult {
    pub manifest: ExtensionManifest,
    pub files: Vec<String>,
    pub warnings: Vec<String>,
}

/// Extension manifest (nova.toml).
#[derive(Debug, Deserialize)]
pub struct ExtensionManifest {
    pub name: String,
    pub title: String,
    pub description: String,
    pub version: String,
    #[serde(default)]
    pub author: Option<String>,
    #[serde(default)]
    pub repo: Option<String>,
    #[serde(default)]
    pub homepage: Option<String>,
    #[serde(default)]
    pub license: Option<String>,
    #[serde(default)]
    pub icon: Option<String>,
    #[serde(default)]
    pub keywords: Vec<String>,
    #[serde(default)]
    pub nova_version: Option<String>,
    #[serde(default)]
    pub changelog: Option<String>,
}

/// Scan an extension package for security issues and parse manifest.
pub fn scan_extension(data: &[u8]) -> Result<ScanResult, ApiError> {
    // Check size limit
    if data.len() > MAX_PACKAGE_SIZE {
        return Err(ApiError::BadRequest(format!(
            "Package too large: {} bytes (max {} bytes)",
            data.len(),
            MAX_PACKAGE_SIZE
        )));
    }

    // Decompress gzip
    let mut decoder = GzDecoder::new(data);
    let mut decompressed = Vec::new();
    decoder.read_to_end(&mut decompressed).map_err(|e| {
        ApiError::BadRequest(format!("Failed to decompress package: {}", e))
    })?;

    // Read tar archive
    let mut archive = Archive::new(decompressed.as_slice());
    let mut manifest: Option<ExtensionManifest> = None;
    let mut files = Vec::new();
    let mut warnings = Vec::new();

    for entry in archive.entries().map_err(|e| {
        ApiError::BadRequest(format!("Invalid tar archive: {}", e))
    })? {
        let mut entry = entry.map_err(|e| {
            ApiError::BadRequest(format!("Failed to read tar entry: {}", e))
        })?;

        let path = entry.path().map_err(|e| {
            ApiError::BadRequest(format!("Invalid path in archive: {}", e))
        })?;
        let path_str = path.to_string_lossy().to_string();

        // Security check: no absolute paths
        if path_str.starts_with('/') {
            return Err(ApiError::BadRequest(
                "Package contains absolute paths".to_string(),
            ));
        }

        // Security check: no path traversal
        if path_str.contains("..") {
            return Err(ApiError::BadRequest(
                "Package contains path traversal".to_string(),
            ));
        }

        files.push(path_str.clone());

        // Parse manifest
        if path_str == "nova.toml" || path_str.ends_with("/nova.toml") {
            let mut content = String::new();
            entry.read_to_string(&mut content).map_err(|e| {
                ApiError::BadRequest(format!("Failed to read manifest: {}", e))
            })?;

            manifest = Some(toml::from_str(&content).map_err(|e| {
                ApiError::BadRequest(format!("Invalid manifest: {}", e))
            })?);
        }

        // Check for suspicious patterns in JS files
        if path_str.ends_with(".js") || path_str.ends_with(".ts") {
            let mut content = String::new();
            entry.read_to_string(&mut content).ok();

            // Check for hardcoded secrets (basic patterns)
            let suspicious_patterns = [
                "AKIA",           // AWS access key prefix
                "-----BEGIN",     // Private key
                "password",       // Might be a secret
                "secret",         // Might be a secret
                "api_key",        // API key variable
                "apikey",         // API key variable
            ];

            for pattern in &suspicious_patterns {
                if content.to_lowercase().contains(&pattern.to_lowercase()) {
                    warnings.push(format!(
                        "File {} may contain sensitive data (pattern: {})",
                        path_str, pattern
                    ));
                }
            }

            // Check for dangerous operations
            let dangerous_patterns = [
                "child_process",
                "require('fs')",
                "require(\"fs\")",
                "eval(",
                "Function(",
            ];

            for pattern in &dangerous_patterns {
                if content.contains(pattern) {
                    warnings.push(format!(
                        "File {} uses potentially dangerous pattern: {}",
                        path_str, pattern
                    ));
                }
            }
        }
    }

    let manifest = manifest.ok_or_else(|| {
        ApiError::BadRequest("Package missing nova.toml manifest".to_string())
    })?;

    // Validate manifest
    validate_manifest(&manifest)?;

    Ok(ScanResult {
        manifest,
        files,
        warnings,
    })
}

/// Validate manifest fields.
fn validate_manifest(manifest: &ExtensionManifest) -> Result<(), ApiError> {
    // Name validation: lowercase alphanumeric with hyphens
    if !manifest.name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '-') {
        return Err(ApiError::BadRequest(
            "Extension name must be lowercase alphanumeric with hyphens only".to_string(),
        ));
    }

    if manifest.name.len() < 2 || manifest.name.len() > 64 {
        return Err(ApiError::BadRequest(
            "Extension name must be 2-64 characters".to_string(),
        ));
    }

    // Title validation
    if manifest.title.is_empty() || manifest.title.len() > 128 {
        return Err(ApiError::BadRequest(
            "Extension title must be 1-128 characters".to_string(),
        ));
    }

    // Description validation
    if manifest.description.is_empty() || manifest.description.len() > 1000 {
        return Err(ApiError::BadRequest(
            "Extension description must be 1-1000 characters".to_string(),
        ));
    }

    // Version validation (semver)
    semver::Version::parse(&manifest.version).map_err(|_| {
        ApiError::BadRequest(format!(
            "Invalid version '{}': must be valid semver (e.g., 1.0.0)",
            manifest.version
        ))
    })?;

    // Keywords validation
    if manifest.keywords.len() > 10 {
        return Err(ApiError::BadRequest(
            "Maximum 10 keywords allowed".to_string(),
        ));
    }

    Ok(())
}

/// Calculate SHA-256 checksum.
pub fn calculate_checksum(data: &[u8]) -> String {
    let mut hasher = Sha256::new();
    hasher.update(data);
    hex::encode(hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_checksum() {
        let data = b"hello world";
        let checksum = calculate_checksum(data);
        assert_eq!(
            checksum,
            "b94d27b9934d3e08a52e52d7da7dabfac484efe37a5380ee9088f7ace2efcde9"
        );
    }
}
