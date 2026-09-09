use rumqttc::{AsyncClient, Event, MqttOptions, Packet, QoS};
use serde::Deserialize;
use std::time::Duration;
use tokio::time;


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


#[tokio::main]
async fn main() {
    println!("Starting Industrial Intelligence Ingestion Service...");

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
                                println!("State: {}", data.state);
                                println!("Temperature: {:.2} °C", data.temperature);
                                println!("RPM: {:.2}", data.rpm);
                                println!("Vibration: {:.2} mm/s", data.vibration);
                                println!("Pressure: {:.2} bar", data.pressure);
                                println!("Power: {:.2} kW", data.power);
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
                                println!("State: {}", data.state);
                                println!("Temperature: {:.2} °C", data.temperature);
                                println!("RPM: {:.2}", data.rpm);
                                println!("Current: {:.2} A", data.current);
                                println!("Vibration: {:.2} mm/s", data.vibration);
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
                                println!("State: {}", data.state);
                                println!("Level: {:.2} %", data.level);
                                println!("Temperature: {:.2} °C", data.temperature);
                                println!("Pressure: {:.2} bar", data.pressure);
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
                                println!("State: {}", data.state);
                                println!("Open: {}", data.open);
                                println!("Flow: {:.2}", data.flow);
                                println!("Position: {:.2} %", data.position);
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