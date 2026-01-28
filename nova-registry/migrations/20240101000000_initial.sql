-- Initial schema for Nova Extension Registry

-- Publishers (developer accounts)
CREATE TABLE publishers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR(64) UNIQUE NOT NULL,
    email VARCHAR(255) UNIQUE NOT NULL,
    github_id BIGINT UNIQUE,
    github_username VARCHAR(64),
    verified BOOLEAN DEFAULT FALSE,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL
);

-- Extensions
CREATE TABLE extensions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    publisher_id UUID NOT NULL REFERENCES publishers(id) ON DELETE CASCADE,
    name VARCHAR(64) NOT NULL,
    title VARCHAR(128) NOT NULL,
    description TEXT NOT NULL,
    icon_url TEXT,
    repo_url TEXT,
    homepage TEXT,
    license VARCHAR(32),
    keywords TEXT[] DEFAULT '{}',
    nova_version VARCHAR(32),
    downloads BIGINT DEFAULT 0 NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    updated_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    UNIQUE(publisher_id, name)
);

-- Extension versions
CREATE TABLE versions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    extension_id UUID NOT NULL REFERENCES extensions(id) ON DELETE CASCADE,
    version VARCHAR(32) NOT NULL,
    download_url TEXT NOT NULL,
    checksum_sha256 VARCHAR(64) NOT NULL,
    changelog TEXT,
    size_bytes BIGINT,
    published_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    UNIQUE(extension_id, version)
);

-- API tokens
CREATE TABLE api_tokens (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    publisher_id UUID NOT NULL REFERENCES publishers(id) ON DELETE CASCADE,
    token_hash VARCHAR(64) NOT NULL,
    name VARCHAR(64) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW() NOT NULL,
    last_used_at TIMESTAMPTZ,
    expires_at TIMESTAMPTZ
);

-- Indexes for search
CREATE INDEX idx_extensions_search ON extensions
    USING GIN (to_tsvector('english', title || ' ' || description));
CREATE INDEX idx_extensions_keywords ON extensions USING GIN (keywords);
CREATE INDEX idx_extensions_downloads ON extensions (downloads DESC);
CREATE INDEX idx_extensions_publisher ON extensions (publisher_id);
CREATE INDEX idx_versions_extension ON versions (extension_id);
CREATE INDEX idx_versions_published ON versions (published_at DESC);
