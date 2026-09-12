use anyhow::{Context, Result};
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tokio::sync::mpsc;

use crate::models::NetworkEvent;

const DATABASE_URL: &str =
    "postgres://industrial:industrial123@localhost:5432/industrial_intelligence";

pub async fn init_pool() -> Result<PgPool> {
    let pool = PgPoolOptions::new()
        .max_connections(5)
        .connect(DATABASE_URL)
        .await
        .context("Failed to connect to TimescaleDB")?;

    sqlx::query(
        r#"
        CREATE TABLE IF NOT EXISTS network_events (
            id              BIGSERIAL PRIMARY KEY,
            time            TIMESTAMPTZ NOT NULL DEFAULT NOW(),
            source_ip       TEXT NOT NULL,
            destination_ip  TEXT NOT NULL,
            source_port     INTEGER NOT NULL,
            destination_port INTEGER NOT NULL,
            transport_protocol TEXT NOT NULL,
            application_protocol TEXT,
            payload_size    INTEGER NOT NULL,
            modbus_function TEXT,
            opcua_message   TEXT,
            suspicious      BOOLEAN NOT NULL DEFAULT FALSE,
            severity        TEXT,
            message         TEXT
        )
        "#,
    )
    .execute(&pool)
    .await?;

    sqlx::query(
        r#"
        CREATE INDEX IF NOT EXISTS idx_network_events_time
        ON network_events (time DESC)
        "#,
    )
    .execute(&pool)
    .await?;

    println!("Connected to TimescaleDB (network_events ready).");
    Ok(pool)
}

pub fn spawn_writer(pool: PgPool) -> mpsc::Sender<NetworkEvent> {
    let (tx, mut rx) = mpsc::channel::<NetworkEvent>(256);

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            if let Err(error) = persist(&pool, &event).await {
                eprintln!("Failed to persist network event: {error}");
            }
        }
    });

    tx
}

async fn persist(pool: &PgPool, event: &NetworkEvent) -> Result<()> {
    let modbus_function = event.modbus.as_ref().map(|m| m.function_name.clone());
    let opcua_message = event.opcua.as_ref().map(|o| o.message_type.clone());

    sqlx::query(
        r#"
        INSERT INTO network_events (
            time,
            source_ip,
            destination_ip,
            source_port,
            destination_port,
            transport_protocol,
            application_protocol,
            payload_size,
            modbus_function,
            opcua_message,
            suspicious,
            severity,
            message
        )
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13)
        "#,
    )
    .bind(event.timestamp)
    .bind(&event.source_ip)
    .bind(&event.destination_ip)
    .bind(event.source_port as i32)
    .bind(event.destination_port as i32)
    .bind(&event.transport_protocol)
    .bind(&event.application_protocol)
    .bind(event.payload_size as i32)
    .bind(modbus_function)
    .bind(opcua_message)
    .bind(event.suspicious)
    .bind(&event.severity)
    .bind(&event.message)
    .execute(pool)
    .await?;

    if event.suspicious {
        create_incident(pool, event).await?;
        create_alert(pool, event).await?;
    }

    Ok(())
}

async fn create_incident(pool: &PgPool, event: &NetworkEvent) -> Result<()> {
    let title = event
        .message
        .clone()
        .unwrap_or_else(|| "OT Security Event".to_string());

    let description = format!(
        "Source {} → {}:{}\nProtocol: {}\n{}",
        event.source_ip,
        event.destination_ip,
        event.destination_port,
        event.application_protocol.as_deref().unwrap_or("unknown"),
        event
            .modbus
            .as_ref()
            .map(|m| format!("Modbus: {}", m.function_name))
            .or_else(|| {
                event
                    .opcua
                    .as_ref()
                    .map(|o| format!("OPC UA: {}", o.message_type))
            })
            .unwrap_or_default()
    );

    sqlx::query(
        r#"
        INSERT INTO incidents (
            created_at,
            severity,
            status,
            title,
            asset_id,
            description
        )
        VALUES (
            NOW(),
            $1,
            'open',
            $2,
            $3,
            $4
        )
        "#,
    )
    .bind(event.severity.as_deref().unwrap_or("HIGH"))
    .bind(&title)
    .bind("PLC-01")
    .bind(&description)
    .execute(pool)
    .await?;

    Ok(())
}

async fn create_alert(pool: &PgPool, event: &NetworkEvent) -> Result<()> {
    let message = event
        .message
        .clone()
        .unwrap_or_else(|| "Suspicious OT network activity".to_string());

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
            'trondheim',
            'PLC-01',
            $1,
            'ot_security',
            $2,
            false
        )
        "#,
    )
    .bind(event.severity.as_deref().unwrap_or("HIGH"))
    .bind(&message)
    .execute(pool)
    .await?;

    Ok(())
}
