//! Admin API handlers.

use axum::{
    extract::{Path, State},
    Json,
};
use serde::Serialize;
use std::sync::Arc;
use uuid::Uuid;

use super::ApiError;
use crate::{db, AppState};

/// Verify a publisher (admin only).
///
/// In production, this would check for admin authentication.
/// For now, it's protected by being an undocumented endpoint.
pub async fn verify_publisher(
    State(state): State<Arc<AppState>>,
    Path(id): Path<Uuid>,
) -> Result<Json<VerifyResponse>, ApiError> {
    // TODO: Add admin authentication

    let publisher = db::get_publisher_by_id(&state.db, id)
        .await?
        .ok_or_else(|| ApiError::NotFound("Publisher not found".to_string()))?;

    db::verify_publisher(&state.db, id).await?;

    Ok(Json(VerifyResponse {
        id,
        name: publisher.name,
        verified: true,
    }))
}

#[derive(Serialize)]
pub struct VerifyResponse {
    pub id: Uuid,
    pub name: String,
    pub verified: bool,
}
