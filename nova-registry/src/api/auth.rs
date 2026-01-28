//! Authentication handlers.

use axum::{
    extract::{Query, State},
    response::{IntoResponse, Redirect},
    Json,
};
use chrono::{Duration, Utc};
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use uuid::Uuid;

use super::{ApiError, AuthenticatedPublisher};
use crate::{db, AppState};

/// JWT claims.
#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: Uuid,
    pub exp: i64,
    pub iat: i64,
}

/// Create a JWT token for a publisher.
pub fn create_jwt(publisher_id: Uuid, secret: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let now = Utc::now();
    let claims = Claims {
        sub: publisher_id,
        exp: (now + Duration::days(30)).timestamp(),
        iat: now.timestamp(),
    };

    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
}

/// Verify a JWT token.
pub fn verify_jwt(token: &str, secret: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &Validation::default(),
    )?;
    Ok(token_data.claims)
}

/// GitHub OAuth login redirect.
#[derive(Deserialize)]
pub struct GitHubLoginParams {
    pub redirect_uri: Option<String>,
}

pub async fn github_login(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GitHubLoginParams>,
) -> impl IntoResponse {
    let redirect_uri = params
        .redirect_uri
        .unwrap_or_else(|| "http://localhost:8080/auth/github/callback".to_string());

    let auth_url = format!(
        "https://github.com/login/oauth/authorize?client_id={}&redirect_uri={}&scope=read:user%20user:email",
        state.github_client_id,
        urlencoding::encode(&redirect_uri)
    );

    Redirect::to(&auth_url)
}

/// GitHub OAuth callback.
#[derive(Deserialize)]
pub struct GitHubCallbackParams {
    pub code: String,
}

#[derive(Deserialize)]
struct GitHubTokenResponse {
    access_token: String,
}

#[derive(Deserialize)]
struct GitHubUser {
    id: i64,
    login: String,
    email: Option<String>,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub publisher: PublisherResponse,
}

#[derive(Serialize)]
pub struct PublisherResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub verified: bool,
}

pub async fn github_callback(
    State(state): State<Arc<AppState>>,
    Query(params): Query<GitHubCallbackParams>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Exchange code for access token
    let client = reqwest::Client::new();
    let token_response: GitHubTokenResponse = client
        .post("https://github.com/login/oauth/access_token")
        .header("Accept", "application/json")
        .form(&[
            ("client_id", state.github_client_id.as_str()),
            ("client_secret", state.github_client_secret.as_str()),
            ("code", params.code.as_str()),
        ])
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("GitHub API error: {}", e)))?
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to parse token response: {}", e)))?;

    // Get user info
    let user_response = client
        .get("https://api.github.com/user")
        .header("Authorization", format!("Bearer {}", token_response.access_token))
        .header("User-Agent", "Nova-Registry")
        .send()
        .await
        .map_err(|e| ApiError::Internal(format!("GitHub API error: {}", e)))?;

    let user: GitHubUser = user_response
        .json()
        .await
        .map_err(|e| ApiError::Internal(format!("Failed to parse user response: {}", e)))?;

    // Get user email if not public
    let email = if let Some(email) = user.email {
        email
    } else {
        // Fetch from emails endpoint
        #[derive(Deserialize)]
        struct GitHubEmail {
            email: String,
            primary: bool,
        }

        let emails_response = client
            .get("https://api.github.com/user/emails")
            .header("Authorization", format!("Bearer {}", token_response.access_token))
            .header("User-Agent", "Nova-Registry")
            .send()
            .await
            .map_err(|e| ApiError::Internal(format!("GitHub API error: {}", e)))?;

        let emails: Vec<GitHubEmail> = emails_response
            .json()
            .await
            .map_err(|e| ApiError::Internal(format!("Failed to parse emails: {}", e)))?;

        emails
            .into_iter()
            .find(|e| e.primary)
            .map(|e| e.email)
            .ok_or_else(|| ApiError::BadRequest("No primary email found".to_string()))?
    };

    // Find or create publisher
    let publisher = match db::get_publisher_by_github_id(&state.db, user.id).await? {
        Some(p) => p,
        None => {
            // Create new publisher
            db::create_publisher(&state.db, &user.login, &email, Some(user.id), Some(&user.login))
                .await?
        }
    };

    // Create JWT
    let token = create_jwt(publisher.id, &state.jwt_secret)
        .map_err(|e| ApiError::Internal(format!("Failed to create token: {}", e)))?;

    Ok(Json(AuthResponse {
        token,
        publisher: PublisherResponse {
            id: publisher.id,
            name: publisher.name,
            email: publisher.email,
            verified: publisher.verified,
        },
    }))
}

/// Create a new API token.
#[derive(Deserialize)]
pub struct CreateTokenRequest {
    pub name: String,
}

pub async fn create_token(
    State(state): State<Arc<AppState>>,
    auth: AuthenticatedPublisher,
    Json(req): Json<CreateTokenRequest>,
) -> Result<Json<AuthResponse>, ApiError> {
    // Create a new JWT
    let token = create_jwt(auth.id, &state.jwt_secret)
        .map_err(|e| ApiError::Internal(format!("Failed to create token: {}", e)))?;

    let publisher = db::get_publisher_by_id(&state.db, auth.id)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    Ok(Json(AuthResponse {
        token,
        publisher: PublisherResponse {
            id: publisher.id,
            name: publisher.name,
            email: publisher.email,
            verified: publisher.verified,
        },
    }))
}

/// Get current user info.
pub async fn me(
    State(state): State<Arc<AppState>>,
    auth: AuthenticatedPublisher,
) -> Result<Json<PublisherResponse>, ApiError> {
    let publisher = db::get_publisher_by_id(&state.db, auth.id)
        .await?
        .ok_or(ApiError::Unauthorized)?;

    Ok(Json(PublisherResponse {
        id: publisher.id,
        name: publisher.name,
        email: publisher.email,
        verified: publisher.verified,
    }))
}
