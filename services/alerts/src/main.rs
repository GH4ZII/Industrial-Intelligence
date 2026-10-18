//! Alarm Engine (Fase 5)
//!
//! Leser siste telemetri fra TimescaleDB og oppretter alarmer ved terskelbrudd.

mod rules;

use std::collections::HashMap;
use std::time::Duration;

use sqlx::{postgres::PgPoolOptions, PgPool};
use tokio::time;

use rules::{evaluate_asset, FiredAlert, LatestMetric};

const POLL_INTERVAL_SECS: u64 = 5;

fn database_url() -> String {
    std::env::var("DATABASE_URL").unwrap_or_else(|_| {
        "postgres://industrial:industrial123@localhost:5432/industrial_intelligence"
            .to_string()
    })
}

async fn connect_db() -> PgPool {
    let url = database_url();
    for attempt in 1..=30 {
        match PgPoolOptions::new().max_connections(5).connect(&url).await {
            Ok(pool) => return pool,
            Err(error) => {
                eprintln!("TimescaleDB connect attempt {attempt}/30 failed: {error}");
                time::sleep(Duration::from_secs(2)).await;
            }
        }
    }
    panic!("Failed to connect to TimescaleDB");
}

#[tokio::main]
async fn main() {
    println!("Starting Industrial Intelligence Alarm Engine...");

    let pool = connect_db().await;

    println!("Connected to TimescaleDB.");
    println!("Rules loaded: temperature, vibration, pressure, RPM deviation");
    println!("Polling every {POLL_INTERVAL_SECS}s");

    loop {
        if let Err(error) = evaluate_rules(&pool).await {
            eprintln!("Alarm check failed: {error}");
        }

        time::sleep(Duration::from_secs(POLL_INTERVAL_SECS)).await;
    }
}

async fn evaluate_rules(pool: &PgPool) -> Result<(), sqlx::Error> {
    let rows = sqlx::query_as::<_, LatestMetric>(
        r#"
        SELECT DISTINCT ON (asset_id, metric)
            site,
            asset_type,
            asset_id,
            metric,
            value,
            time
        FROM telemetry
        ORDER BY asset_id, metric, time DESC
        "#,
    )
    .fetch_all(pool)
    .await?;

    let mut by_asset: HashMap<String, Vec<&LatestMetric>> = HashMap::new();
    for row in &rows {
        by_asset.entry(row.asset_id.clone()).or_default().push(row);
    }

    for (_asset_id, metrics) in by_asset {
        for alert in evaluate_asset(metrics) {
            let inserted = insert_alert_if_new(
                pool,
                &alert.site,
                &alert.asset_id,
                alert.severity,
                alert.alert_type,
                &alert.message,
            )
            .await?;

            if inserted {
                print_alert(&alert);
            }
        }
    }

    Ok(())
}

fn print_alert(alert: &FiredAlert) {
    println!("--------------------------------");
    println!("{}", alert.severity);
    println!();
    println!("{}", alert.asset_id);
    println!();
    println!("{}", alert.message);
    println!();
    println!("{:.1} {}", alert.value, alert.unit);
    println!();
    println!("Detected:");
    println!("{}", alert.detected_at.format("%H:%M:%S"));
    println!("--------------------------------");
}

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
