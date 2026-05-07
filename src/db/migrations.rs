pub const SCHEMA_V1: &str = "
    CREATE TABLE IF NOT EXISTS engagements (
        id          TEXT PRIMARY KEY,
        name        TEXT NOT NULL UNIQUE,
        description TEXT,
        created_at  TEXT NOT NULL,
        updated_at  TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS targets (
        id            TEXT PRIMARY KEY,
        engagement_id TEXT NOT NULL REFERENCES engagements(id) ON DELETE CASCADE,
        domain        TEXT NOT NULL,
        created_at    TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS subdomains (
        id            TEXT PRIMARY KEY,
        target_id     TEXT NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
        subdomain     TEXT NOT NULL,
        status        TEXT NOT NULL DEFAULT 'not-visited',
        notes         TEXT,
        status_code   INTEGER,
        title         TEXT,
        created_at    TEXT NOT NULL,
        updated_at    TEXT NOT NULL,
        UNIQUE(target_id, subdomain)
    );

    CREATE TABLE IF NOT EXISTS technologies (
        id           TEXT PRIMARY KEY,
        subdomain_id TEXT NOT NULL REFERENCES subdomains(id) ON DELETE CASCADE,
        name         TEXT NOT NULL,
        version      TEXT,
        created_at   TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS tags (
        id           TEXT PRIMARY KEY,
        subdomain_id TEXT NOT NULL REFERENCES subdomains(id) ON DELETE CASCADE,
        name         TEXT NOT NULL,
        created_at   TEXT NOT NULL,
        UNIQUE(subdomain_id, name)
    );

    CREATE TABLE IF NOT EXISTS urls (
        id           TEXT PRIMARY KEY,
        subdomain_id TEXT NOT NULL REFERENCES subdomains(id) ON DELETE CASCADE,
        url          TEXT NOT NULL,
        url_type     TEXT NOT NULL DEFAULT 'unknown',
        created_at   TEXT NOT NULL,
        UNIQUE(subdomain_id, url)
    );

    CREATE TABLE IF NOT EXISTS ips (
        id         TEXT PRIMARY KEY,
        target_id  TEXT NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
        ip         TEXT NOT NULL,
        created_at TEXT NOT NULL,
        UNIQUE(target_id, ip)
    );

    CREATE TABLE IF NOT EXISTS asns (
        id         TEXT PRIMARY KEY,
        target_id  TEXT NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
        asn        TEXT NOT NULL,
        org        TEXT,
        created_at TEXT NOT NULL,
        UNIQUE(target_id, asn)
    );

    CREATE TABLE IF NOT EXISTS screenshots (
        id           TEXT PRIMARY KEY,
        subdomain_id TEXT NOT NULL REFERENCES subdomains(id) ON DELETE CASCADE,
        file_path    TEXT NOT NULL,
        created_at   TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS scan_history (
        id         TEXT PRIMARY KEY,
        target_id  TEXT NOT NULL REFERENCES targets(id) ON DELETE CASCADE,
        source     TEXT NOT NULL,
        records    INTEGER NOT NULL DEFAULT 0,
        imported_at TEXT NOT NULL
    );

    CREATE TABLE IF NOT EXISTS schema_version (
        version INTEGER PRIMARY KEY
    );

    INSERT OR IGNORE INTO schema_version (version) VALUES (1);
";
