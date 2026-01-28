//! Registry client for CLI commands.
//!
//! Handles communication with the Nova extension registry.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

/// Default registry URL.
pub const DEFAULT_REGISTRY_URL: &str = "https://registry.nova.dev";

/// Extension info from registry.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionInfo {
    pub publisher: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub downloads: i64,
    pub latest_version: Option<String>,
    pub updated_at: String,
}

/// Update info from registry.
#[derive(Debug, Deserialize)]
pub struct UpdateInfo {
    pub name: String,
    pub current: String,
    pub latest: String,
    pub changelog: Option<String>,
}

/// Updates response from registry.
#[derive(Debug, Deserialize)]
pub struct UpdatesResponse {
    pub available: Vec<UpdateInfo>,
}

/// Publish response from registry.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PublishResponse {
    pub publisher: String,
    pub name: String,
    pub version: String,
    pub download_url: String,
}

/// Get the registry URL from environment or default.
pub fn registry_url() -> String {
    std::env::var("NOVA_REGISTRY_URL").unwrap_or_else(|_| DEFAULT_REGISTRY_URL.to_string())
}

/// Search extensions in the registry.
pub async fn search(query: &str, limit: usize) -> Result<Vec<ExtensionInfo>> {
    let url = format!(
        "{}/api/extensions/search?q={}&limit={}",
        registry_url(),
        urlencoding::encode(query),
        limit
    );

    let response = reqwest::get(&url)
        .await
        .context("Failed to connect to registry")?;

    if !response.status().is_success() {
        anyhow::bail!("Registry error: {}", response.status());
    }

    let extensions: Vec<ExtensionInfo> = response
        .json()
        .await
        .context("Failed to parse registry response")?;

    Ok(extensions)
}

/// Get extension info from registry.
pub async fn get_extension(publisher: &str, name: &str) -> Result<Option<ExtensionInfo>> {
    let url = format!("{}/api/extensions/{}/{}", registry_url(), publisher, name);

    let response = reqwest::get(&url)
        .await
        .context("Failed to connect to registry")?;

    if response.status().as_u16() == 404 {
        return Ok(None);
    }

    if !response.status().is_success() {
        anyhow::bail!("Registry error: {}", response.status());
    }

    let extension: ExtensionInfo = response
        .json()
        .await
        .context("Failed to parse registry response")?;

    Ok(Some(extension))
}

/// Download extension package from registry.
pub async fn download(publisher: &str, name: &str, version: Option<&str>) -> Result<Vec<u8>> {
    let url = if let Some(v) = version {
        format!(
            "{}/api/extensions/{}/{}/versions/{}/download",
            registry_url(),
            publisher,
            name,
            v
        )
    } else {
        format!(
            "{}/api/extensions/{}/{}/download",
            registry_url(),
            publisher,
            name
        )
    };

    let response = reqwest::get(&url)
        .await
        .context("Failed to download extension")?;

    if !response.status().is_success() {
        anyhow::bail!("Download failed: {}", response.status());
    }

    let bytes = response.bytes().await.context("Failed to read download")?;

    Ok(bytes.to_vec())
}

/// Check for updates.
pub async fn check_updates(installed: &[(String, String)]) -> Result<Vec<UpdateInfo>> {
    if installed.is_empty() {
        return Ok(Vec::new());
    }

    let installed_param = installed
        .iter()
        .map(|(name, version)| format!("{}@{}", name, version))
        .collect::<Vec<_>>()
        .join(",");

    let url = format!(
        "{}/api/updates?installed={}",
        registry_url(),
        urlencoding::encode(&installed_param)
    );

    let response = reqwest::get(&url)
        .await
        .context("Failed to check for updates")?;

    if !response.status().is_success() {
        anyhow::bail!("Failed to check updates: {}", response.status());
    }

    let updates: UpdatesResponse = response.json().await.context("Failed to parse updates")?;

    Ok(updates.available)
}

/// Publish extension to registry.
pub async fn publish(path: &Path, token: &str) -> Result<PublishResponse> {
    // Build the extension first
    super::build::run_build(&path.to_string_lossy())?;

    // Create tarball
    let tarball = create_tarball(path)?;

    // Upload to registry
    let client = reqwest::Client::new();
    let url = format!("{}/api/extensions", registry_url());

    let form = reqwest::multipart::Form::new().part(
        "package",
        reqwest::multipart::Part::bytes(tarball)
            .file_name("extension.tar.gz")
            .mime_str("application/gzip")?,
    );

    let response = client
        .post(&url)
        .bearer_auth(token)
        .multipart(form)
        .send()
        .await
        .context("Failed to upload to registry")?;

    if !response.status().is_success() {
        let error_text = response.text().await.unwrap_or_default();
        anyhow::bail!("Publish failed: {}", error_text);
    }

    let result: PublishResponse = response
        .json()
        .await
        .context("Failed to parse publish response")?;

    Ok(result)
}

/// Create a tarball from extension directory.
fn create_tarball(path: &Path) -> Result<Vec<u8>> {
    use flate2::write::GzEncoder;
    use flate2::Compression;
    use tar::Builder;

    let mut encoder = GzEncoder::new(Vec::new(), Compression::default());

    {
        let mut builder = Builder::new(&mut encoder);

        // Add dist directory
        let dist_path = path.join("dist");
        if dist_path.exists() {
            builder
                .append_dir_all("dist", &dist_path)
                .context("Failed to add dist directory")?;
        }

        // Add nova.toml
        let manifest_path = path.join("nova.toml");
        if manifest_path.exists() {
            builder
                .append_path_with_name(&manifest_path, "nova.toml")
                .context("Failed to add manifest")?;
        } else {
            anyhow::bail!("nova.toml not found in extension directory");
        }

        builder.finish()?;
    }

    let tarball = encoder.finish()?;
    Ok(tarball)
}

/// Get auth token from config or prompt.
pub fn get_auth_token() -> Result<String> {
    // Try environment variable first
    if let Ok(token) = std::env::var("NOVA_REGISTRY_TOKEN") {
        return Ok(token);
    }

    // Try config file
    let config_path = dirs::config_dir().map(|p| p.join("nova").join("registry-token"));

    if let Some(path) = config_path {
        if path.exists() {
            let token = std::fs::read_to_string(&path)
                .context("Failed to read auth token")?
                .trim()
                .to_string();
            return Ok(token);
        }
    }

    anyhow::bail!(
        "No registry auth token found.\n\n\
         To authenticate, run:\n  \
         nova login\n\n\
         Or set NOVA_REGISTRY_TOKEN environment variable."
    )
}

/// Save auth token to config.
pub fn save_auth_token(token: &str) -> Result<()> {
    let config_dir = dirs::config_dir()
        .context("Could not find config directory")?
        .join("nova");

    std::fs::create_dir_all(&config_dir)?;

    let token_path = config_dir.join("registry-token");
    std::fs::write(&token_path, token)?;

    // Set restrictive permissions on Unix
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&token_path, std::fs::Permissions::from_mode(0o600))?;
    }

    Ok(())
}
