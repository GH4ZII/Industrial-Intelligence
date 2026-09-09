use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::Deserialize;
use std::time::Duration;
use tokio::time;
use sqlx::{PgPool, postgres::PgPoolOptions};


#[derive(Debug, Deserialize)]
struct PumpTelemetry {
    id: String,
    state: String,
    rpm: f64,
    temperature: f64,
    vibration: f64,
    pressure: f64,
    power: f64,
}


#[derive(Debug, Deserialize)]
struct MotorTelemetry {
    id: String,
    state: String,
    temperature: f64,
    rpm: f64,
    current: f64,
    vibration: f64,
}


#[derive(Debug, Deserialize)]
struct TankTelemetry {
    id: String,
    state: String,
    level: f64,
    temperature: f64,
    pressure: f64,
}


#[derive(Debug, Deserialize)]
struct ValveTelemetry {
    id: String,
    state: String,
    open: bool,
    flow: f64,
    position: f64,
}

async fn insert_metric(
    pool: &PgPool,
    site: &str,
    asset_type: &str,
    asset_id: &str,
    metric: &str,
    value: f64,
) -> Result<(), sqlx::Error> {
    sqlx::query(
        r#"
        INSERT INTO telemetry (
            time,
            site,
            asset_type,
            asset_id,
            metric,
            value
        )
        VALUES (
            NOW(),
            $1,
            $2,
            $3,
            $4,
            $5
        )
        "#,
    )
    .bind(site)
    .bind(asset_type)
    .bind(asset_id)
    .bind(metric)
    .bind(value)
    .execute(pool)
    .await?;

    Ok(())
}

#[tokio::main]
async fn main() {
    println!("Starting Industrial Intelligence Ingestion Service...");

    let database_url =
    "postgres://industrial:industrial123@localhost:5432/industrial_intelligence";

let pool = PgPoolOptions::new()
    .max_connections(5)
    .connect(database_url)
    .await
    .expect("Failed to connect to TimescaleDB");

println!("Connected to TimescaleDB.");

    let mut mqtt_options =
        MqttOptions::new("ingestion-service", "localhost", 1883);

    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let (mqtt_client, mut eventloop) =
        AsyncClient::new(mqtt_options, 10);

    mqtt_client
        .subscribe(
            "factory/trondheim/+/+/telemetry",
            QoS::AtLeastOnce,
        )
        .await
        .expect("Failed to subscribe");

    println!("Subscribed to factory telemetry.");
    println!("Waiting for messages...\n");


    loop {
        match eventloop.poll().await {

            Ok(Event::Incoming(Packet::Publish(message))) => {

                let topic = message.topic;

                let payload =
                    String::from_utf8_lossy(&message.payload);


                // -------------------------
                // Parse MQTT topic
                // -------------------------

                let parts: Vec<&str> =
                    topic.split('/').collect();


                if parts.len() != 5 {
                    eprintln!("Invalid topic: {}", topic);
                    continue;
                }


                let site = parts[1];
                let asset_type = parts[2];
                let asset_id = parts[3];


                println!("--------------------------------");

                println!("Site: {}", site);
                println!("Asset Type: {}", asset_type);
                println!("Asset ID: {}", asset_id);


                // -------------------------
                // Parse JSON
                // -------------------------

                match asset_type {

                    "pump" => {

                        match serde_json::from_str::<PumpTelemetry>(&payload) {

                            Ok(data) => {
                                println!("Received telemetry from {}", data.id);
                            
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "temperature",
                                    data.temperature,
                                )
                                .await
                                .expect("Failed to insert temperature");
                            
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "rpm",
                                    data.rpm,
                                )
                                .await
                                .expect("Failed to insert RPM");
                            
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "vibration",
                                    data.vibration,
                                )
                                .await
                                .expect("Failed to insert vibration");
                            
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "pressure",
                                    data.pressure,
                                )
                                .await
                                .expect("Failed to insert pressure");
                            
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "power",
                                    data.power,
                                )
                                .await
                                .expect("Failed to insert power");
                            
                                println!("Stored P101 telemetry in TimescaleDB.");
                            }

                            Err(error) => {
                                eprintln!(
                                    "Failed to parse pump JSON: {}",
                                    error
                                );
                            }
                        }
                    }


                    "motor" => {

                        match serde_json::from_str::<MotorTelemetry>(&payload) {

                            Ok(data) => {
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "temperature",
                                    data.temperature,
                                )
                                .await
                                .unwrap();
                            
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "rpm",
                                    data.rpm,
                                )
                                .await
                                .unwrap();
                            
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "current",
                                    data.current,
                                )
                                .await
                                .unwrap();
                            
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "vibration",
                                    data.vibration,
                                )
                                .await
                                .unwrap();
                            
                                println!("Stored M201 telemetry in TimescaleDB.");
                            }

                            Err(error) => {
                                eprintln!(
                                    "Failed to parse motor JSON: {}",
                                    error
                                );
                            }
                        }
                    }


                    "tank" => {

                        match serde_json::from_str::<TankTelemetry>(&payload) {

                            Ok(data) => {
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "level",
                                    data.level,
                                )
                                .await
                                .unwrap();
                            
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "temperature",
                                    data.temperature,
                                )
                                .await
                                .unwrap();
                            
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "pressure",
                                    data.pressure,
                                )
                                .await
                                .unwrap();
                            
                                println!("Stored T301 telemetry in TimescaleDB.");
                            }

                            Err(error) => {
                                eprintln!(
                                    "Failed to parse tank JSON: {}",
                                    error
                                );
                            }
                        }
                    }


                    "valve" => {

                        match serde_json::from_str::<ValveTelemetry>(&payload) {

                            Ok(data) => {
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "flow",
                                    data.flow,
                                )
                                .await
                                .unwrap();
                            
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "position",
                                    data.position,
                                )
                                .await
                                .unwrap();
                            
                                insert_metric(
                                    &pool,
                                    site,
                                    asset_type,
                                    asset_id,
                                    "open",
                                    if data.open { 1.0 } else { 0.0 },
                                )
                                .await
                                .unwrap();
                            
                                println!("Stored V401 telemetry in TimescaleDB.");
                            }

                            Err(error) => {
                                eprintln!(
                                    "Failed to parse valve JSON: {}",
                                    error
                                );
                            }
                        }
                    }


                    _ => {
                        eprintln!(
                            "Unknown asset type: {}",
                            asset_type
                        );
                    }
                }


                println!("--------------------------------\n");
            }


            Ok(_) => {}


            Err(error) => {

                eprintln!("MQTT error: {:?}", error);

                time::sleep(
                    Duration::from_secs(1)
                ).await;
            }
        }
    }
}