//! API handlers.

pub mod admin;
pub mod auth;
pub mod extensions;
pub mod publishers;

use axum::{
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use std::sync::Arc;

use crate::AppState;

/// API error type.
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Forbidden: {0}")]
    Forbidden(String),

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error("Internal error: {0}")]
    Internal(String),
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            ApiError::NotFound(m) => (StatusCode::NOT_FOUND, m.clone()),
            ApiError::BadRequest(m) => (StatusCode::BAD_REQUEST, m.clone()),
            ApiError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            ApiError::Forbidden(m) => (StatusCode::FORBIDDEN, m.clone()),
            ApiError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            ApiError::Internal(m) => (StatusCode::INTERNAL_SERVER_ERROR, m.clone()),
        };

        let body = Json(ErrorResponse { error: message });
        (status, body).into_response()
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        tracing::error!("Database error: {:?}", err);
        ApiError::Internal("Database error".to_string())
    }
}

/// Authenticated publisher extracted from request.
pub struct AuthenticatedPublisher {
    pub id: uuid::Uuid,
    pub name: String,
    pub verified: bool,
}

#[axum::async_trait]
impl FromRequestParts<Arc<AppState>> for AuthenticatedPublisher {
    type Rejection = ApiError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &Arc<AppState>,
    ) -> Result<Self, Self::Rejection> {
        // Extract Authorization header
        let auth_header = parts
            .headers
            .get("Authorization")
            .and_then(|v| v.to_str().ok())
            .ok_or(ApiError::Unauthorized)?;

        // Parse Bearer token
        let token = auth_header
            .strip_prefix("Bearer ")
            .ok_or(ApiError::Unauthorized)?;

        // Verify JWT
        let claims = auth::verify_jwt(token, &state.jwt_secret).map_err(|_| ApiError::Unauthorized)?;

        // Get publisher from database
        let publisher = crate::db::get_publisher_by_id(&state.db, claims.sub)
            .await?
            .ok_or(ApiError::Unauthorized)?;

        Ok(AuthenticatedPublisher {
            id: publisher.id,
            name: publisher.name,
            verified: publisher.verified,
        })
    }
}
