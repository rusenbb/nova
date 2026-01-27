//! Extension browser service for in-app registry browsing.
//!
//! This module handles communication with the extension registry for
//! the in-app extension browser.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// Default registry URL.
pub const REGISTRY_URL: &str = "https://registry.nova.dev";

/// Get the registry URL from environment or default.
pub fn registry_url() -> String {
    std::env::var("NOVA_REGISTRY_URL").unwrap_or_else(|_| REGISTRY_URL.to_string())
}

/// Browser tab for the extension browser view.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum BrowserTab {
    #[default]
    Discover,
    Installed,
    Updates,
}

/// Extension card displayed in the browser.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionCard {
    pub publisher: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub downloads: u64,
    pub installed: bool,
    pub installed_version: Option<String>,
    pub update_available: Option<String>,
}

/// Extension browser data for the UI.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionBrowserData {
    pub extensions: Vec<ExtensionCard>,
    pub search_query: String,
    pub loading: bool,
    pub tab: BrowserTab,
    pub error: Option<String>,
}

/// Extension detail for the detail view.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExtensionDetail {
    pub publisher: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub repo_url: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub downloads: u64,
    pub latest_version: String,
    pub updated_at: String,
    pub installed: bool,
    pub installed_version: Option<String>,
    pub commands: Vec<CommandInfo>,
}

/// Command info for extension detail.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CommandInfo {
    pub name: String,
    pub title: String,
    pub subtitle: Option<String>,
}

/// Registry extension info (API response).
#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RegistryExtension {
    pub publisher: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub downloads: i64,
    pub latest_version: Option<String>,
    pub updated_at: String,
}

/// Update info for installed extensions.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateAvailable {
    pub name: String,
    pub current_version: String,
    pub latest_version: String,
    pub changelog: Option<String>,
}

/// Extension browser client.
pub struct BrowserClient {
    base_url: String,
}

impl BrowserClient {
    /// Create a new browser client.
    pub fn new() -> Self {
        Self {
            base_url: registry_url(),
        }
    }

    /// Search extensions in the registry.
    pub async fn search(&self, query: &str, limit: usize) -> Result<Vec<RegistryExtension>> {
        let url = format!(
            "{}/api/extensions/search?q={}&limit={}",
            self.base_url,
            urlencoding::encode(query),
            limit
        );

        let response = reqwest::get(&url)
            .await
            .context("Failed to connect to registry")?;

        if !response.status().is_success() {
            anyhow::bail!("Registry error: {}", response.status());
        }

        let extensions: Vec<RegistryExtension> = response
            .json()
            .await
            .context("Failed to parse registry response")?;

        Ok(extensions)
    }

    /// Get popular extensions (for Discover tab).
    pub async fn get_popular(&self, limit: usize) -> Result<Vec<RegistryExtension>> {
        let url = format!("{}/api/extensions?limit={}", self.base_url, limit);

        let response = reqwest::get(&url)
            .await
            .context("Failed to connect to registry")?;

        if !response.status().is_success() {
            anyhow::bail!("Registry error: {}", response.status());
        }

        let extensions: Vec<RegistryExtension> = response
            .json()
            .await
            .context("Failed to parse registry response")?;

        Ok(extensions)
    }

    /// Get extension detail.
    pub async fn get_extension(
        &self,
        publisher: &str,
        name: &str,
    ) -> Result<Option<ExtensionDetail>> {
        let url = format!("{}/api/extensions/{}/{}", self.base_url, publisher, name);

        let response = reqwest::get(&url)
            .await
            .context("Failed to connect to registry")?;

        if response.status().as_u16() == 404 {
            return Ok(None);
        }

        if !response.status().is_success() {
            anyhow::bail!("Registry error: {}", response.status());
        }

        let detail: ExtensionDetail = response
            .json()
            .await
            .context("Failed to parse extension detail")?;

        Ok(Some(detail))
    }

    /// Check for updates.
    pub async fn check_updates(
        &self,
        installed: &[(String, String)],
    ) -> Result<Vec<UpdateAvailable>> {
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
            self.base_url,
            urlencoding::encode(&installed_param)
        );

        let response = reqwest::get(&url)
            .await
            .context("Failed to check for updates")?;

        if !response.status().is_success() {
            anyhow::bail!("Failed to check updates: {}", response.status());
        }

        #[derive(Deserialize)]
        struct UpdatesResponse {
            available: Vec<UpdateAvailable>,
        }

        let updates: UpdatesResponse = response
            .json()
            .await
            .context("Failed to parse updates response")?;

        Ok(updates.available)
    }
}

impl Default for BrowserClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Build extension browser data.
pub fn build_browser_data(
    registry_extensions: &[RegistryExtension],
    installed: &[(String, String)],
    updates: &[UpdateAvailable],
    tab: BrowserTab,
    search_query: &str,
) -> ExtensionBrowserData {
    let installed_set: std::collections::HashSet<_> =
        installed.iter().map(|(n, _)| n.as_str()).collect();
    let installed_versions: std::collections::HashMap<_, _> = installed
        .iter()
        .map(|(n, v)| (n.as_str(), v.as_str()))
        .collect();
    let update_versions: std::collections::HashMap<_, _> = updates
        .iter()
        .map(|u| (u.name.as_str(), u.latest_version.as_str()))
        .collect();

    let extensions: Vec<ExtensionCard> = registry_extensions
        .iter()
        .map(|ext| {
            let full_name = format!("{}/{}", ext.publisher, ext.name);
            let is_installed = installed_set.contains(full_name.as_str());

            ExtensionCard {
                publisher: ext.publisher.clone(),
                name: ext.name.clone(),
                title: ext.title.clone(),
                description: ext.description.clone(),
                icon_url: ext.icon_url.clone(),
                downloads: ext.downloads as u64,
                installed: is_installed,
                installed_version: installed_versions
                    .get(full_name.as_str())
                    .map(|s| s.to_string()),
                update_available: update_versions
                    .get(full_name.as_str())
                    .map(|s| s.to_string()),
            }
        })
        .collect();

    ExtensionBrowserData {
        extensions,
        search_query: search_query.to_string(),
        loading: false,
        tab,
        error: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_browser_tab_serialize() {
        let tab = BrowserTab::Discover;
        let json = serde_json::to_string(&tab).unwrap();
        assert_eq!(json, "\"discover\"");

        let tab = BrowserTab::Updates;
        let json = serde_json::to_string(&tab).unwrap();
        assert_eq!(json, "\"updates\"");
    }

    #[test]
    fn test_extension_card_serialize() {
        let card = ExtensionCard {
            publisher: "test".to_string(),
            name: "hello".to_string(),
            title: "Hello World".to_string(),
            description: "A test extension".to_string(),
            icon_url: None,
            downloads: 100,
            installed: false,
            installed_version: None,
            update_available: None,
        };

        let json = serde_json::to_string(&card).unwrap();
        assert!(json.contains("\"publisher\":\"test\""));
        assert!(json.contains("\"iconUrl\":null"));
    }

    #[test]
    fn test_build_browser_data() {
        let registry = vec![RegistryExtension {
            publisher: "test".to_string(),
            name: "hello".to_string(),
            title: "Hello".to_string(),
            description: "Test".to_string(),
            icon_url: None,
            downloads: 10,
            latest_version: Some("1.0.0".to_string()),
            updated_at: "2024-01-01".to_string(),
        }];

        let installed = vec![("test/hello".to_string(), "0.9.0".to_string())];
        let updates = vec![UpdateAvailable {
            name: "test/hello".to_string(),
            current_version: "0.9.0".to_string(),
            latest_version: "1.0.0".to_string(),
            changelog: None,
        }];

        let data = build_browser_data(&registry, &installed, &updates, BrowserTab::Discover, "");

        assert_eq!(data.extensions.len(), 1);
        assert!(data.extensions[0].installed);
        assert_eq!(
            data.extensions[0].installed_version,
            Some("0.9.0".to_string())
        );
        assert_eq!(
            data.extensions[0].update_available,
            Some("1.0.0".to_string())
        );
    }
}
