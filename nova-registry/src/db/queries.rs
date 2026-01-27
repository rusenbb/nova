//! Database queries.
//!
//! Uses runtime query checking to avoid needing a database connection at compile time.

use sqlx::PgPool;
use uuid::Uuid;

use super::models::*;

/// Create a new publisher.
pub async fn create_publisher(
    pool: &PgPool,
    name: &str,
    email: &str,
    github_id: Option<i64>,
    github_username: Option<&str>,
) -> sqlx::Result<Publisher> {
    sqlx::query_as::<_, Publisher>(
        r#"
        INSERT INTO publishers (name, email, github_id, github_username)
        VALUES ($1, $2, $3, $4)
        RETURNING id, name, email, github_id, github_username, verified, created_at
        "#,
    )
    .bind(name)
    .bind(email)
    .bind(github_id)
    .bind(github_username)
    .fetch_one(pool)
    .await
}

/// Get a publisher by GitHub ID.
pub async fn get_publisher_by_github_id(
    pool: &PgPool,
    github_id: i64,
) -> sqlx::Result<Option<Publisher>> {
    sqlx::query_as::<_, Publisher>(
        r#"SELECT id, name, email, github_id, github_username, verified, created_at
           FROM publishers WHERE github_id = $1"#,
    )
    .bind(github_id)
    .fetch_optional(pool)
    .await
}

/// Get a publisher by name.
pub async fn get_publisher_by_name(pool: &PgPool, name: &str) -> sqlx::Result<Option<Publisher>> {
    sqlx::query_as::<_, Publisher>(
        r#"SELECT id, name, email, github_id, github_username, verified, created_at
           FROM publishers WHERE name = $1"#,
    )
    .bind(name)
    .fetch_optional(pool)
    .await
}

/// Get a publisher by ID.
pub async fn get_publisher_by_id(pool: &PgPool, id: Uuid) -> sqlx::Result<Option<Publisher>> {
    sqlx::query_as::<_, Publisher>(
        r#"SELECT id, name, email, github_id, github_username, verified, created_at
           FROM publishers WHERE id = $1"#,
    )
    .bind(id)
    .fetch_optional(pool)
    .await
}

/// Verify a publisher.
pub async fn verify_publisher(pool: &PgPool, id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE publishers SET verified = true WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Search extensions.
pub async fn search_extensions(
    pool: &PgPool,
    query: &str,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<ExtensionWithPublisher>> {
    // Use full-text search for query
    let search_query = query
        .split_whitespace()
        .map(|w| format!("{}:*", w))
        .collect::<Vec<_>>()
        .join(" & ");

    sqlx::query_as::<_, ExtensionWithPublisher>(
        r#"
        SELECT
            e.id,
            p.name as publisher,
            e.name,
            e.title,
            e.description,
            e.icon_url,
            e.repo_url,
            e.homepage,
            e.license,
            e.keywords,
            e.nova_version,
            e.downloads,
            (SELECT version FROM versions WHERE extension_id = e.id ORDER BY published_at DESC LIMIT 1) as latest_version,
            e.created_at,
            e.updated_at
        FROM extensions e
        JOIN publishers p ON e.publisher_id = p.id
        WHERE to_tsvector('english', e.title || ' ' || e.description) @@ to_tsquery('english', $1)
           OR e.name ILIKE '%' || $2 || '%'
           OR e.title ILIKE '%' || $2 || '%'
           OR $2 = ANY(e.keywords)
        ORDER BY e.downloads DESC
        LIMIT $3 OFFSET $4
        "#,
    )
    .bind(&search_query)
    .bind(query)
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// List extensions with pagination.
pub async fn list_extensions(
    pool: &PgPool,
    limit: i64,
    offset: i64,
) -> sqlx::Result<Vec<ExtensionWithPublisher>> {
    sqlx::query_as::<_, ExtensionWithPublisher>(
        r#"
        SELECT
            e.id,
            p.name as publisher,
            e.name,
            e.title,
            e.description,
            e.icon_url,
            e.repo_url,
            e.homepage,
            e.license,
            e.keywords,
            e.nova_version,
            e.downloads,
            (SELECT version FROM versions WHERE extension_id = e.id ORDER BY published_at DESC LIMIT 1) as latest_version,
            e.created_at,
            e.updated_at
        FROM extensions e
        JOIN publishers p ON e.publisher_id = p.id
        ORDER BY e.downloads DESC
        LIMIT $1 OFFSET $2
        "#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(pool)
    .await
}

/// Get an extension by publisher name and extension name.
pub async fn get_extension(
    pool: &PgPool,
    publisher: &str,
    name: &str,
) -> sqlx::Result<Option<ExtensionWithPublisher>> {
    sqlx::query_as::<_, ExtensionWithPublisher>(
        r#"
        SELECT
            e.id,
            p.name as publisher,
            e.name,
            e.title,
            e.description,
            e.icon_url,
            e.repo_url,
            e.homepage,
            e.license,
            e.keywords,
            e.nova_version,
            e.downloads,
            (SELECT version FROM versions WHERE extension_id = e.id ORDER BY published_at DESC LIMIT 1) as latest_version,
            e.created_at,
            e.updated_at
        FROM extensions e
        JOIN publishers p ON e.publisher_id = p.id
        WHERE p.name = $1 AND e.name = $2
        "#,
    )
    .bind(publisher)
    .bind(name)
    .fetch_optional(pool)
    .await
}

/// Get extension versions.
pub async fn get_extension_versions(
    pool: &PgPool,
    extension_id: Uuid,
) -> sqlx::Result<Vec<VersionInfo>> {
    sqlx::query_as::<_, VersionInfo>(
        r#"
        SELECT version, changelog, size_bytes, published_at
        FROM versions
        WHERE extension_id = $1
        ORDER BY published_at DESC
        "#,
    )
    .bind(extension_id)
    .fetch_all(pool)
    .await
}

/// Get latest version for an extension.
pub async fn get_latest_version(
    pool: &PgPool,
    extension_id: Uuid,
) -> sqlx::Result<Option<ExtensionVersion>> {
    sqlx::query_as::<_, ExtensionVersion>(
        r#"
        SELECT id, extension_id, version, download_url, checksum_sha256, changelog, size_bytes, published_at
        FROM versions
        WHERE extension_id = $1
        ORDER BY published_at DESC
        LIMIT 1
        "#,
    )
    .bind(extension_id)
    .fetch_optional(pool)
    .await
}

/// Get specific version.
pub async fn get_version(
    pool: &PgPool,
    extension_id: Uuid,
    version: &str,
) -> sqlx::Result<Option<ExtensionVersion>> {
    sqlx::query_as::<_, ExtensionVersion>(
        r#"
        SELECT id, extension_id, version, download_url, checksum_sha256, changelog, size_bytes, published_at
        FROM versions
        WHERE extension_id = $1 AND version = $2
        "#,
    )
    .bind(extension_id)
    .bind(version)
    .fetch_optional(pool)
    .await
}

/// Increment download count.
pub async fn increment_downloads(pool: &PgPool, extension_id: Uuid) -> sqlx::Result<()> {
    sqlx::query("UPDATE extensions SET downloads = downloads + 1 WHERE id = $1")
        .bind(extension_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Create or update an extension.
pub async fn upsert_extension(
    pool: &PgPool,
    publisher_id: Uuid,
    name: &str,
    title: &str,
    description: &str,
    icon_url: Option<&str>,
    repo_url: Option<&str>,
    homepage: Option<&str>,
    license: Option<&str>,
    keywords: &[String],
    nova_version: Option<&str>,
) -> sqlx::Result<Extension> {
    sqlx::query_as::<_, Extension>(
        r#"
        INSERT INTO extensions (publisher_id, name, title, description, icon_url, repo_url, homepage, license, keywords, nova_version)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
        ON CONFLICT (publisher_id, name) DO UPDATE SET
            title = EXCLUDED.title,
            description = EXCLUDED.description,
            icon_url = EXCLUDED.icon_url,
            repo_url = EXCLUDED.repo_url,
            homepage = EXCLUDED.homepage,
            license = EXCLUDED.license,
            keywords = EXCLUDED.keywords,
            nova_version = EXCLUDED.nova_version,
            updated_at = NOW()
        RETURNING id, publisher_id, name, title, description, icon_url, repo_url, homepage, license, keywords, nova_version, downloads, created_at, updated_at
        "#,
    )
    .bind(publisher_id)
    .bind(name)
    .bind(title)
    .bind(description)
    .bind(icon_url)
    .bind(repo_url)
    .bind(homepage)
    .bind(license)
    .bind(keywords)
    .bind(nova_version)
    .fetch_one(pool)
    .await
}

/// Create a new version.
pub async fn create_version(
    pool: &PgPool,
    extension_id: Uuid,
    version: &str,
    download_url: &str,
    checksum: &str,
    changelog: Option<&str>,
    size_bytes: Option<i64>,
) -> sqlx::Result<ExtensionVersion> {
    sqlx::query_as::<_, ExtensionVersion>(
        r#"
        INSERT INTO versions (extension_id, version, download_url, checksum_sha256, changelog, size_bytes)
        VALUES ($1, $2, $3, $4, $5, $6)
        RETURNING id, extension_id, version, download_url, checksum_sha256, changelog, size_bytes, published_at
        "#,
    )
    .bind(extension_id)
    .bind(version)
    .bind(download_url)
    .bind(checksum)
    .bind(changelog)
    .bind(size_bytes)
    .fetch_one(pool)
    .await
}

/// Delete an extension.
pub async fn delete_extension(pool: &PgPool, extension_id: Uuid) -> sqlx::Result<()> {
    // Versions are deleted by cascade
    sqlx::query("DELETE FROM extensions WHERE id = $1")
        .bind(extension_id)
        .execute(pool)
        .await?;
    Ok(())
}

/// Check for updates.
pub async fn check_updates(
    pool: &PgPool,
    installed: &[(String, String)], // (name, version) pairs
) -> sqlx::Result<Vec<UpdateInfo>> {
    let mut updates = Vec::new();

    for (name, current_version) in installed {
        // Parse publisher/name format
        let parts: Vec<&str> = name.split('/').collect();
        if parts.len() != 2 {
            continue;
        }

        let (publisher, ext_name) = (parts[0], parts[1]);

        if let Some(ext) = get_extension(pool, publisher, ext_name).await? {
            if let Some(ref latest) = ext.latest_version {
                // Compare versions using semver
                let current = semver::Version::parse(current_version).ok();
                let latest_v = semver::Version::parse(latest).ok();

                if let (Some(c), Some(l)) = (current, latest_v) {
                    if l > c {
                        // Get changelog for latest version
                        let versions = get_extension_versions(pool, ext.id).await?;
                        let changelog = versions.first().and_then(|v| v.changelog.clone());

                        updates.push(UpdateInfo {
                            name: name.clone(),
                            current: current_version.clone(),
                            latest: latest.clone(),
                            changelog,
                        });
                    }
                }
            }
        }
    }

    Ok(updates)
}
