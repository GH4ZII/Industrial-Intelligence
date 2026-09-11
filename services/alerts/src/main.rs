//! Alarm Engine – første del.
//!
//! Leser siste temperatur per asset fra TimescaleDB.
//! Hvis temperature > 90 °C, opprettes en WARNING i alerts-tabellen.
//! (Kjøres hvert 5. sekund; unngår duplikater innen siste 5 minutter.)

use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use std::time::Duration;
use tokio::time;

const DATABASE_URL: &str =
    "postgres://industrial:industrial123@localhost:5432/industrial_intelligence";

const TEMPERATURE_WARNING: f64 = 90.0;
const POLL_INTERVAL_SECS: u64 = 5;

#[derive(Debug, FromRow)]
struct LatestMetric {
    site: String,
    asset_id: String,
    value: f64,
    time: DateTime<Utc>,
}

#[tokio::main]
async fn main() {
    println!("Starting Industrial Intelligence Alarm Engine...");

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(DATABASE_URL)
        .await
        .expect("Failed to connect to TimescaleDB");

    println!("Connected to TimescaleDB.");
    println!(
        "Rule: temperature > {} °C → WARNING (every {}s)",
        TEMPERATURE_WARNING, POLL_INTERVAL_SECS
    );

    loop {
        if let Err(error) = evaluate_temperature_warnings(&pool).await {
            eprintln!("Alarm check failed: {}", error);
        }

        time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

async fn evaluate_temperature_warnings(pool: &PgPool) -> Result<(), sqlx::Error> {
    let rows = sqlx::query_as::<_, LatestMetric>(
        r#"
        SELECT DISTINCT ON (asset_id)
            site,
            asset_id,
            value,
            time
        FROM telemetry
        WHERE metric = 'temperature'
        ORDER BY asset_id, time DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    for row in rows {
        if row.value <= TEMPERATURE_WARNING {
            continue;
        }

        let message = format!(
            "High temperature detected: {:.1} °C (threshold {} °C)",
            row.value, TEMPERATURE_WARNING
        );

        let inserted = insert_alert_if_new(
            pool,
            &row.site,
            &row.asset_id,
            "WARNING",
            "temperature_high",
            &message,
        )
        .await?;

        if inserted {
            println!(
                "ALERT {} {} — {:.1} °C @ {}",
                row.asset_id, "WARNING", row.value, row.time
            );
        }
    }

    Ok(())
}

/// Inserts an alert unless the same asset+type already has an
/// unacknowledged alert within the last DEDUP_MINUTES.
async fn insert_alert_if_new(
    pool: &PgPool,
    site: &str,
    asset_id: &str,
    severity: &str,
    alert_type: &str,
    message: &str,
) -> Result<bool, sqlx::Error> {
    let existing: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM alerts
        WHERE asset_id = $1
          AND alert_type = $2
          AND acknowledged = false
          AND time > NOW() - INTERVAL '5 minutes'
        "#,
    )
    .bind(asset_id)
    .bind(alert_type)
    .fetch_one(pool)
    .await?;

    if existing.0 > 0 {
        return Ok(false);
    }

    sqlx::query(
        r#"
        INSERT INTO alerts (
            time,
            site,
            asset_id,
            severity,
            alert_type,
            message,
            acknowledged
        )
        VALUES (
            NOW(),
            $1,
            $2,
            $3,
            $4,
            $5,
            false
        )
        "#,
    )
    .bind(site)
    .bind(asset_id)
    .bind(severity)
    .bind(alert_type)
    .bind(message)
    .execute(pool)
    .await?;

    Ok(true)
}
