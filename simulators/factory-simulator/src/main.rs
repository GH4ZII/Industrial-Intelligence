mod assets;
mod factory;

use factory::Factory;
use std::time::Duration;
use rumqttc::{AsyncClient, MqttOptions, QoS};
use tokio::time;

#[tokio::main]
async fn main() {
    // 1. Opprett fabrikken
    let mut factory = Factory::new();

    // 2. MQTT-konfigurasjon
    let mut mqtt_options =
        MqttOptions::new("factory-simulator", "localhost", 1883);

    mqtt_options.set_keep_alive(Duration::from_secs(5));

    // 3. Opprett MQTT-client
    let (mqtt_client, mut eventloop) =
        AsyncClient::new(mqtt_options, 10);

    // 4. Start MQTT event loop
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

    // 5. Kjør fabrikken kontinuerlig
    loop {
        factory.tick();

        let json = factory.to_json();

        println!("{}\n", json);

        // 6. Publiser fabrikkdata til MQTT
        mqtt_client
            .publish(
                "factory/trondheim/telemetry",
                QoS::AtLeastOnce,
                false,
                json,
            )
            .await
            .unwrap();

        // 7. Vent ett sekund
        time::sleep(Duration::from_secs(1)).await;
    }
}