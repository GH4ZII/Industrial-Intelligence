mod assets;
mod factory;

use factory::Factory;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use std::time::Duration;
use tokio::time;

#[tokio::main]
async fn main() {
    let mut factory = Factory::new();

    // MQTT
    let mut mqtt_options =
        MqttOptions::new("factory-simulator", "localhost", 1883);

    mqtt_options.set_keep_alive(Duration::from_secs(5));

    let (mqtt_client, mut eventloop) =
        AsyncClient::new(mqtt_options, 10);

    // MQTT event loop
    tokio::spawn(async move {
        loop {
            match eventloop.poll().await {
                Ok(_) => {}
                Err(error) => {
                    eprintln!("MQTT error: {:?}", error);

                    time::sleep(Duration::from_secs(1)).await;
                }
            }
        }
    });

    // Factory simulation
    loop {
        factory.tick();

        // Gjør hver maskin om til JSON
        let pump_json =
            serde_json::to_string(factory.pump()).unwrap();

        let motor_json =
            serde_json::to_string(factory.motor()).unwrap();

        let tank_json =
            serde_json::to_string(factory.tank()).unwrap();

        let valve_json =
            serde_json::to_string(factory.valve()).unwrap();

        // Pumpe
        mqtt_client
            .publish(
                "factory/trondheim/pump/P101/telemetry",
                QoS::AtLeastOnce,
                false,
                pump_json,
            )
            .await
            .expect("Failed to publish pump");

        // Motor
        mqtt_client
            .publish(
                "factory/trondheim/motor/M201/telemetry",
                QoS::AtLeastOnce,
                false,
                motor_json,
            )
            .await
            .expect("Failed to publish motor");

        // Tank
        mqtt_client
            .publish(
                "factory/trondheim/tank/T301/telemetry",
                QoS::AtLeastOnce,
                false,
                tank_json,
            )
            .await
            .expect("Failed to publish tank");

        // Valve
        mqtt_client
            .publish(
                "factory/trondheim/valve/V401/telemetry",
                QoS::AtLeastOnce,
                false,
                valve_json,
            )
            .await
            .expect("Failed to publish valve");

        println!("Published factory telemetry");

        time::sleep(Duration::from_secs(1)).await;
    }
}