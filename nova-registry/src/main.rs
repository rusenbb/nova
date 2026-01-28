//! Nova Extension Registry Server
//!
//! A central registry for discovering, publishing, and distributing
//! Nova extensions.

mod api;
mod db;
mod scanning;
mod storage;

use axum::{
    routing::{delete, get, post, put},
    Router,
};
use sqlx::postgres::PgPoolOptions;
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

/// Application state shared across handlers.
#[derive(Clone)]
pub struct AppState {
    pub db: sqlx::PgPool,
    pub s3: storage::S3Client,
    pub jwt_secret: String,
    pub github_client_id: String,
    pub github_client_secret: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "nova_registry=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Connect to database
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgres://localhost/nova_registry".to_string());

    tracing::info!("Connecting to database...");
    let db = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await?;

    // Run migrations
    tracing::info!("Running database migrations...");
    sqlx::migrate!("./migrations").run(&db).await?;

    // Initialize S3 client
    let s3 = storage::S3Client::from_env().await?;

    // Build application state
    let state = Arc::new(AppState {
        db,
        s3,
        jwt_secret: std::env::var("JWT_SECRET").unwrap_or_else(|_| "development-secret".into()),
        github_client_id: std::env::var("GITHUB_CLIENT_ID").unwrap_or_default(),
        github_client_secret: std::env::var("GITHUB_CLIENT_SECRET").unwrap_or_default(),
    });

    // Build router
    let app = Router::new()
        // Public API
        .route("/api/extensions", get(api::extensions::list))
        .route("/api/extensions/search", get(api::extensions::search))
        .route("/api/extensions/:publisher/:name", get(api::extensions::get))
        .route(
            "/api/extensions/:publisher/:name/versions",
            get(api::extensions::versions),
        )
        .route(
            "/api/extensions/:publisher/:name/download",
            get(api::extensions::download),
        )
        .route(
            "/api/extensions/:publisher/:name/versions/:version/download",
            get(api::extensions::download_version),
        )
        .route("/api/updates", get(api::extensions::check_updates))
        // Authenticated API
        .route("/api/extensions", post(api::extensions::publish))
        .route(
            "/api/extensions/:publisher/:name",
            delete(api::extensions::unpublish),
        )
        // Auth
        .route("/auth/github", get(api::auth::github_login))
        .route("/auth/github/callback", get(api::auth::github_callback))
        .route("/auth/token", post(api::auth::create_token))
        .route("/auth/me", get(api::auth::me))
        // Publishers
        .route("/api/publishers/:name", get(api::publishers::get))
        .route("/api/publishers/:name", put(api::publishers::update))
        // Admin
        .route(
            "/admin/publishers/:id/verify",
            post(api::admin::verify_publisher),
        )
        // Health check
        .route("/health", get(|| async { "OK" }))
        // Middleware
        .layer(TraceLayer::new_for_http())
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        )
        .with_state(state);

    // Start server
    let port: u16 = std::env::var("PORT")
        .ok()
        .and_then(|p| p.parse().ok())
        .unwrap_or(8080);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Starting server on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
