//! Alarm Engine (Fase 5)
//!
//! Leser siste telemetri fra TimescaleDB og oppretter alarmer ved terskelbrudd.
//!
//! Regler:
//! - temperature > 90  → WARNING
//! - temperature > 110 → CRITICAL
//! - vibration > 5     → WARNING
//! - vibration > 9     → CRITICAL
//! - pressure < 3.5    → WARNING
//! - pressure < 2.5    → CRITICAL
//! - RPM avvik > 20% fra nominal → WARNING
//! - RPM avvik > 30% fra nominal → CRITICAL

use chrono::{DateTime, Utc};
use sqlx::{postgres::PgPoolOptions, FromRow, PgPool};
use std::collections::HashMap;
use std::time::Duration;
use tokio::time;

const DATABASE_URL: &str =
    "postgres://industrial:industrial123@localhost:5432/industrial_intelligence";

const POLL_INTERVAL_SECS: u64 = 5;

const TEMP_WARNING: f64 = 90.0;
const TEMP_CRITICAL: f64 = 110.0;
const VIB_WARNING: f64 = 5.0;
const VIB_CRITICAL: f64 = 9.0;
const PRESSURE_WARNING: f64 = 3.5;
const PRESSURE_CRITICAL: f64 = 2.5;
const RPM_DEV_WARNING: f64 = 0.20;
const RPM_DEV_CRITICAL: f64 = 0.30;

#[derive(Debug, FromRow)]
struct LatestMetric {
    site: String,
    asset_type: String,
    asset_id: String,
    metric: String,
    value: f64,
    time: DateTime<Utc>,
}

#[derive(Debug)]
struct FiredAlert {
    site: String,
    asset_id: String,
    severity: &'static str,
    alert_type: &'static str,
    message: String,
    value: f64,
    unit: &'static str,
    detected_at: DateTime<Utc>,
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
    println!("Rules loaded: temperature, vibration, pressure, RPM deviation");
    println!("Polling every {}s", POLL_INTERVAL_SECS);

    loop {
        if let Err(error) = evaluate_rules(&pool).await {
            eprintln!("Alarm check failed: {}", error);
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

    // Group latest metrics by asset.
    let mut by_asset: HashMap<String, Vec<&LatestMetric>> = HashMap::new();
    for row in &rows {
        by_asset
            .entry(row.asset_id.clone())
            .or_default()
            .push(row);
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

fn evaluate_asset(metrics: Vec<&LatestMetric>) -> Vec<FiredAlert> {
    let mut alerts = Vec::new();

    let site = metrics
        .first()
        .map(|m| m.site.as_str())
        .unwrap_or("unknown");
    let asset_id = metrics
        .first()
        .map(|m| m.asset_id.as_str())
        .unwrap_or("unknown");
    let asset_type = metrics
        .first()
        .map(|m| m.asset_type.as_str())
        .unwrap_or("asset");

    for metric in &metrics {
        match metric.metric.as_str() {
            "temperature" => {
                if let Some((severity, alert_type)) =
                    high_threshold(metric.value, TEMP_WARNING, TEMP_CRITICAL, "temperature")
                {
                    alerts.push(FiredAlert {
                        site: site.to_string(),
                        asset_id: asset_id.to_string(),
                        severity,
                        alert_type,
                        message: format!(
                            "High temperature detected on {} {}",
                            asset_type, asset_id
                        ),
                        value: metric.value,
                        unit: "°C",
                        detected_at: metric.time,
                    });
                }
            }
            "vibration" => {
                if let Some((severity, alert_type)) =
                    high_threshold(metric.value, VIB_WARNING, VIB_CRITICAL, "vibration")
                {
                    alerts.push(FiredAlert {
                        site: site.to_string(),
                        asset_id: asset_id.to_string(),
                        severity,
                        alert_type,
                        message: format!(
                            "High vibration detected on {} {}",
                            asset_type, asset_id
                        ),
                        value: metric.value,
                        unit: "mm/s",
                        detected_at: metric.time,
                    });
                }
            }
            "pressure" => {
                if let Some((severity, alert_type)) =
                    low_threshold(metric.value, PRESSURE_WARNING, PRESSURE_CRITICAL, "pressure")
                {
                    alerts.push(FiredAlert {
                        site: site.to_string(),
                        asset_id: asset_id.to_string(),
                        severity,
                        alert_type,
                        message: format!(
                            "Low pressure detected on {} {}",
                            asset_type, asset_id
                        ),
                        value: metric.value,
                        unit: "bar",
                        detected_at: metric.time,
                    });
                }
            }
            "rpm" => {
                if let Some(nominal) = nominal_rpm(asset_type, asset_id) {
                    let deviation = (metric.value - nominal).abs() / nominal;
                    if let Some((severity, alert_type)) =
                        high_threshold(deviation, RPM_DEV_WARNING, RPM_DEV_CRITICAL, "rpm_deviation")
                    {
                        alerts.push(FiredAlert {
                            site: site.to_string(),
                            asset_id: asset_id.to_string(),
                            severity,
                            alert_type,
                            message: format!(
                                "RPM deviation detected on {} {} ({:.0} vs nominal {:.0})",
                                asset_type, asset_id, metric.value, nominal
                            ),
                            value: deviation * 100.0,
                            unit: "%",
                            detected_at: metric.time,
                        });
                    }
                }
            }
            _ => {}
        }
    }

    alerts
}

fn high_threshold(
    value: f64,
    warning: f64,
    critical: f64,
    base: &'static str,
) -> Option<(&'static str, &'static str)> {
    if value > critical {
        Some(("CRITICAL", match base {
            "temperature" => "temperature_critical",
            "vibration" => "vibration_critical",
            "rpm_deviation" => "rpm_deviation_critical",
            _ => "threshold_critical",
        }))
    } else if value > warning {
        Some(("WARNING", match base {
            "temperature" => "temperature_warning",
            "vibration" => "vibration_warning",
            "rpm_deviation" => "rpm_deviation_warning",
            _ => "threshold_warning",
        }))
    } else {
        None
    }
}

fn low_threshold(
    value: f64,
    warning: f64,
    critical: f64,
    _base: &'static str,
) -> Option<(&'static str, &'static str)> {
    // critical is lower than warning for low-threshold rules
    if value < critical {
        Some(("CRITICAL", "pressure_critical"))
    } else if value < warning {
        Some(("WARNING", "pressure_warning"))
    } else {
        None
    }
}

fn nominal_rpm(asset_type: &str, asset_id: &str) -> Option<f64> {
    match (asset_type, asset_id) {
        ("pump", _) | (_, "P101") => Some(1450.0),
        ("motor", _) | (_, "M201") => Some(1450.0),
        _ => None,
    }
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
