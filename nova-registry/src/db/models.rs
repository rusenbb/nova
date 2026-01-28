//! Database models.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use uuid::Uuid;

/// A publisher (developer account) in the registry.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Publisher {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub github_id: Option<i64>,
    pub github_username: Option<String>,
    pub verified: bool,
    pub created_at: DateTime<Utc>,
}

/// An extension in the registry.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Extension {
    pub id: Uuid,
    pub publisher_id: Uuid,
    pub name: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub repo_url: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub keywords: Vec<String>,
    pub nova_version: Option<String>,
    pub downloads: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A specific version of an extension.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ExtensionVersion {
    pub id: Uuid,
    pub extension_id: Uuid,
    pub version: String,
    pub download_url: String,
    pub checksum_sha256: String,
    pub changelog: Option<String>,
    pub size_bytes: Option<i64>,
    pub published_at: DateTime<Utc>,
}

/// Extension with publisher info for API responses.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct ExtensionWithPublisher {
    pub id: Uuid,
    pub publisher: String,
    pub name: String,
    pub title: String,
    pub description: String,
    pub icon_url: Option<String>,
    pub repo_url: Option<String>,
    pub homepage: Option<String>,
    pub license: Option<String>,
    pub keywords: Vec<String>,
    pub nova_version: Option<String>,
    pub downloads: i64,
    pub latest_version: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Extension detail response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtensionDetail {
    #[serde(flatten)]
    pub extension: ExtensionWithPublisher,
    pub versions: Vec<VersionInfo>,
}

/// Version info for API responses.
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub changelog: Option<String>,
    pub size_bytes: Option<i64>,
    pub published_at: DateTime<Utc>,
}

/// Update check response.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateInfo {
    pub name: String,
    pub current: String,
    pub latest: String,
    pub changelog: Option<String>,
}

/// API token stored in database.
#[derive(Debug, Clone, FromRow)]
pub struct ApiToken {
    pub id: Uuid,
    pub publisher_id: Uuid,
    pub token_hash: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub expires_at: Option<DateTime<Utc>>,
}
