//! Extension API handlers.

use axum::{
    extract::{Multipart, Path, Query, State},
    response::Redirect,
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{ApiError, AuthenticatedPublisher};
use crate::{db, scanning, AppState};

/// List/search query parameters.
#[derive(Deserialize)]
pub struct ListParams {
    pub q: Option<String>,
    pub limit: Option<i64>,
    pub offset: Option<i64>,
}

/// List extensions.
pub async fn list(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<db::ExtensionWithPublisher>>, ApiError> {
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    let extensions = db::list_extensions(&state.db, limit, offset).await?;
    Ok(Json(extensions))
}

/// Search extensions.
pub async fn search(
    State(state): State<Arc<AppState>>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<db::ExtensionWithPublisher>>, ApiError> {
    let query = params.q.unwrap_or_default();
    let limit = params.limit.unwrap_or(20).min(100);
    let offset = params.offset.unwrap_or(0);

    if query.is_empty() {
        let extensions = db::list_extensions(&state.db, limit, offset).await?;
        return Ok(Json(extensions));
    }

    let extensions = db::search_extensions(&state.db, &query, limit, offset).await?;
    Ok(Json(extensions))
}

/// Get extension details.
pub async fn get(
    State(state): State<Arc<AppState>>,
    Path((publisher, name)): Path<(String, String)>,
) -> Result<Json<db::ExtensionDetail>, ApiError> {
    let extension = db::get_extension(&state.db, &publisher, &name)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("{}/{} not found", publisher, name)))?;

    let versions = db::get_extension_versions(&state.db, extension.id).await?;

    Ok(Json(db::ExtensionDetail {
        extension,
        versions,
    }))
}

/// Get extension versions.
pub async fn versions(
    State(state): State<Arc<AppState>>,
    Path((publisher, name)): Path<(String, String)>,
) -> Result<Json<Vec<db::VersionInfo>>, ApiError> {
    let extension = db::get_extension(&state.db, &publisher, &name)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("{}/{} not found", publisher, name)))?;

    let versions = db::get_extension_versions(&state.db, extension.id).await?;
    Ok(Json(versions))
}

/// Download latest version.
pub async fn download(
    State(state): State<Arc<AppState>>,
    Path((publisher, name)): Path<(String, String)>,
) -> Result<Redirect, ApiError> {
    let extension = db::get_extension(&state.db, &publisher, &name)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("{}/{} not found", publisher, name)))?;

    let version = db::get_latest_version(&state.db, extension.id)
        .await?
        .ok_or_else(|| ApiError::NotFound("No versions available".to_string()))?;

    // Increment download count
    db::increment_downloads(&state.db, extension.id).await?;

    Ok(Redirect::to(&version.download_url))
}

/// Download specific version.
pub async fn download_version(
    State(state): State<Arc<AppState>>,
    Path((publisher, name, version)): Path<(String, String, String)>,
) -> Result<Redirect, ApiError> {
    let extension = db::get_extension(&state.db, &publisher, &name)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("{}/{} not found", publisher, name)))?;

    let ver = db::get_version(&state.db, extension.id, &version)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Version {} not found", version)))?;

    // Increment download count
    db::increment_downloads(&state.db, extension.id).await?;

    Ok(Redirect::to(&ver.download_url))
}

/// Check for updates.
#[derive(Deserialize)]
pub struct UpdatesParams {
    pub installed: String, // Comma-separated "name@version" pairs
}

#[derive(Serialize)]
pub struct UpdatesResponse {
    pub available: Vec<db::UpdateInfo>,
}

pub async fn check_updates(
    State(state): State<Arc<AppState>>,
    Query(params): Query<UpdatesParams>,
) -> Result<Json<UpdatesResponse>, ApiError> {
    // Parse installed extensions
    let installed: Vec<(String, String)> = params
        .installed
        .split(',')
        .filter_map(|s| {
            let parts: Vec<&str> = s.trim().split('@').collect();
            if parts.len() == 2 {
                Some((parts[0].to_string(), parts[1].to_string()))
            } else {
                None
            }
        })
        .collect();

    let updates = db::check_updates(&state.db, &installed).await?;
    Ok(Json(UpdatesResponse { available: updates }))
}

/// Publish an extension.
pub async fn publish(
    State(state): State<Arc<AppState>>,
    auth: AuthenticatedPublisher,
    mut multipart: Multipart,
) -> Result<Json<PublishResponse>, ApiError> {
    // Check if publisher is verified (for new extensions)
    // Verified publishers can publish immediately, unverified go to review

    let mut package_data: Option<Vec<u8>> = None;

    while let Some(field) = multipart.next_field().await.map_err(|e| {
        ApiError::BadRequest(format!("Failed to read multipart: {}", e))
    })? {
        let name = field.name().unwrap_or("").to_string();
        if name == "package" {
            package_data = Some(field.bytes().await.map_err(|e| {
                ApiError::BadRequest(format!("Failed to read package: {}", e))
            })?.to_vec());
        }
    }

    let package_data = package_data.ok_or_else(|| {
        ApiError::BadRequest("No package file provided".to_string())
    })?;

    // Scan the package
    let scan_result = scanning::scan_extension(&package_data)?;

    // Parse manifest
    let manifest = scan_result.manifest;

    // Validate version
    let version = semver::Version::parse(&manifest.version).map_err(|_| {
        ApiError::BadRequest(format!("Invalid version: {}", manifest.version))
    })?;

    // Check for existing extension
    if let Some(existing) = db::get_extension(&state.db, &auth.name, &manifest.name).await? {
        // Check if version already exists
        if let Some(_) = db::get_version(&state.db, existing.id, &manifest.version).await? {
            return Err(ApiError::Conflict(format!(
                "Version {} already exists",
                manifest.version
            )));
        }

        // Check that new version is higher
        if let Some(latest) = db::get_latest_version(&state.db, existing.id).await? {
            let latest_v = semver::Version::parse(&latest.version).unwrap_or_else(|_| {
                semver::Version::new(0, 0, 0)
            });
            if version <= latest_v {
                return Err(ApiError::BadRequest(format!(
                    "New version {} must be higher than {}",
                    version, latest_v
                )));
            }
        }
    }

    // Calculate checksum
    let checksum = scanning::calculate_checksum(&package_data);

    // Upload to S3
    let s3_key = format!(
        "extensions/{}/{}/{}.tar.gz",
        auth.name, manifest.name, manifest.version
    );
    let download_url = state.s3.upload(&s3_key, &package_data).await?;

    // Create/update extension in database
    let extension = db::upsert_extension(
        &state.db,
        auth.id,
        &manifest.name,
        &manifest.title,
        &manifest.description,
        manifest.icon.as_deref(),
        manifest.repo.as_deref(),
        manifest.homepage.as_deref(),
        manifest.license.as_deref(),
        &manifest.keywords,
        manifest.nova_version.as_deref(),
    )
    .await?;

    // Create version
    let _version = db::create_version(
        &state.db,
        extension.id,
        &manifest.version,
        &download_url,
        &checksum,
        manifest.changelog.as_deref(),
        Some(package_data.len() as i64),
    )
    .await?;

    Ok(Json(PublishResponse {
        publisher: auth.name,
        name: manifest.name,
        version: manifest.version,
        download_url,
    }))
}

#[derive(Serialize)]
pub struct PublishResponse {
    pub publisher: String,
    pub name: String,
    pub version: String,
    pub download_url: String,
}

/// Unpublish (delete) an extension.
pub async fn unpublish(
    State(state): State<Arc<AppState>>,
    auth: AuthenticatedPublisher,
    Path((publisher, name)): Path<(String, String)>,
) -> Result<Json<DeleteResponse>, ApiError> {
    // Can only delete your own extensions
    if publisher != auth.name {
        return Err(ApiError::Forbidden(
            "Can only delete your own extensions".to_string(),
        ));
    }

    let extension = db::get_extension(&state.db, &publisher, &name)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("{}/{} not found", publisher, name)))?;

    // Delete from database (versions cascade)
    db::delete_extension(&state.db, extension.id).await?;

    // Note: S3 files are not deleted to allow recovery

    Ok(Json(DeleteResponse {
        deleted: format!("{}/{}", publisher, name),
    }))
}

#[derive(Serialize)]
pub struct DeleteResponse {
    pub deleted: String,
}
