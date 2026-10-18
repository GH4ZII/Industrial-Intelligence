use sqlx::PgPool;

/// Ensure tables added after the original init.sql exist on already-created volumes.
pub async fn ensure_schema(pool: &PgPool) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        ALTER TABLE incidents
            ADD COLUMN IF NOT EXISTS updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        ALTER TABLE incidents
            ADD COLUMN IF NOT EXISTS assigned_to TEXT
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        ALTER TABLE incidents
            ADD COLUMN IF NOT EXISTS mitre_id TEXT
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        ALTER TABLE incidents
            ADD COLUMN IF NOT EXISTS mitre_name TEXT
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS incident_events (
            id          BIGSERIAL PRIMARY KEY,
            incident_id BIGINT NOT NULL REFERENCES incidents(id) ON DELETE CASCADE,
            time        TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            message     TEXT NOT NULL
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_incident_events_incident
            ON incident_events (incident_id, time)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS anomaly_scores (
            time                   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            site                   TEXT NOT NULL,
            asset_id               TEXT NOT NULL,
            score                  DOUBLE PRECISION NOT NULL,
            is_anomaly             BOOLEAN NOT NULL DEFAULT FALSE,
            contributing_metrics   TEXT,
            message                TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_anomaly_scores_asset
            ON anomaly_scores (asset_id, time DESC)
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS asset_health (
            time                 TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            site                 TEXT NOT NULL,
            asset_id             TEXT NOT NULL,
            health_percent       DOUBLE PRECISION NOT NULL,
            risk_7d              DOUBLE PRECISION NOT NULL,
            risk_30d             DOUBLE PRECISION NOT NULL,
            risk_90d             DOUBLE PRECISION NOT NULL,
            recommended_action   TEXT
        )
        "#,
    )
    .execute(pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_asset_health_asset
            ON asset_health (asset_id, time DESC)
        "#,
    )
    .execute(pool)
    .await?;

    Ok(())
}

pub fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://industrial:industrial123@localhost:5432/industrial_intelligence"
            .to_string()
    })
}
