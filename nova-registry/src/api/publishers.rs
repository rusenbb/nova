//! Publisher API handlers.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

use super::{ApiError, AuthenticatedPublisher};
use crate::{db, AppState};

/// Publisher public profile.
#[derive(Serialize)]
pub struct PublisherProfile {
    pub name: String,
    pub github_username: Option<String>,
    pub verified: bool,
    pub extension_count: i64,
}

/// Get publisher profile.
pub async fn get(
    State(state): State<Arc<AppState>>,
    Path(name): Path<String>,
) -> Result<Json<PublisherProfile>, ApiError> {
    let publisher = db::get_publisher_by_name(&state.db, &name)
        .await?
        .ok_or_else(|| ApiError::NotFound(format!("Publisher {} not found", name)))?;

    // Count extensions
    let extensions = db::list_extensions(&state.db, 1000, 0).await?;
    let extension_count = extensions
        .iter()
        .filter(|e| e.publisher == publisher.name)
        .count() as i64;

    Ok(Json(PublisherProfile {
        name: publisher.name,
        github_username: publisher.github_username,
        verified: publisher.verified,
        extension_count,
    }))
}

/// Update publisher profile.
#[derive(Deserialize)]
pub struct UpdatePublisherRequest {
    pub email: Option<String>,
}

pub async fn update(
    State(state): State<Arc<AppState>>,
    auth: AuthenticatedPublisher,
    Path(name): Path<String>,
    Json(req): Json<UpdatePublisherRequest>,
) -> Result<Json<PublisherProfile>, ApiError> {
    // Can only update your own profile
    if name != auth.name {
        return Err(ApiError::Forbidden(
            "Can only update your own profile".to_string(),
        ));
    }

    // Update email if provided
    if let Some(email) = req.email {
        sqlx::query("UPDATE publishers SET email = $1 WHERE id = $2")
            .bind(&email)
            .bind(auth.id)
            .execute(&state.db)
            .await?;
    }

    // Return updated profile
    get(State(state), Path(name)).await
}
