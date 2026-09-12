use chrono::Utc;
use tokio::sync::mpsc;

use crate::models::{ModbusEvent, NetworkEvent, OpcUaEvent};

/// Known OT assets (allowed to talk industrial protocols).
const TRUSTED_SOURCES: &[&str] = &[
    "192.168.10.20", // Industrial Gateway
    "192.168.10.30", // HMI
    "192.168.10.40", // Engineering Workstation
];

const PLC_SUBNET_PREFIX: &str = "192.168.10.";

pub fn handle_event(mut event: NetworkEvent, tx: &mpsc::Sender<NetworkEvent>) {
    enrich_suspicion(&mut event);
    print_event(&event);

    if let Err(error) = tx.try_send(event) {
        eprintln!("Event queue full/closed: {error}");
    }
}

fn enrich_suspicion(event: &mut NetworkEvent) {
    let from_unknown = !TRUSTED_SOURCES.iter().any(|ip| *ip == event.source_ip)
        && !event.source_ip.starts_with("127.")
        && event.source_ip != "::1";

    let to_plc = event.destination_ip.starts_with(PLC_SUBNET_PREFIX)
        && event.destination_port == 502;

    if let Some(modbus) = &event.modbus {
        if modbus.is_write && from_unknown && to_plc {
            event.suspicious = true;
            event.severity = Some("HIGH".to_string());
            event.message = Some(format!(
                "Unauthorized Modbus Write ({}) from {} to {}",
                modbus.function_name, event.source_ip, event.destination_ip
            ));
            return;
        }

        if modbus.is_write {
            event.suspicious = true;
            event.severity = Some("MEDIUM".to_string());
            event.message = Some(format!(
                "Modbus write observed: {} → {} ({})",
                event.source_ip, event.destination_ip, modbus.function_name
            ));
            return;
        }
    }

    if event.application_protocol.as_deref() == Some("OPC UA") && from_unknown {
        event.suspicious = true;
        event.severity = Some("MEDIUM".to_string());
        event.message = Some(format!(
            "OPC UA activity from unknown source {}",
            event.source_ip
        ));
    }
}

fn print_event(event: &NetworkEvent) {
    if event.suspicious {
        println!("--------------------------------");
        println!(
            "{} SECURITY ALERT",
            event.severity.as_deref().unwrap_or("HIGH")
        );
        println!();
        if let Some(message) = &event.message {
            println!("{message}");
        }
        println!();
        println!("Source:");
        println!("{}", event.source_ip);
        println!();
        println!("Destination:");
        println!("{}:{}", event.destination_ip, event.destination_port);
        if let Some(modbus) = &event.modbus {
            println!();
            println!("Function:");
            println!("{}", modbus.function_name);
        }
        println!("--------------------------------");
    } else {
        let app = event.application_protocol.as_deref().unwrap_or("unknown");
        println!(
            "[{}] {} {}:{} → {}:{} ({}) {}B",
            event.timestamp.format("%H:%M:%S"),
            event.transport_protocol,
            event.source_ip,
            event.source_port,
            event.destination_ip,
            event.destination_port,
            app,
            event.payload_size
        );
    }
}

/// Synthetic traffic for demos when no real Modbus/OPC UA is on the wire.
pub async fn run_demo_loop(tx: mpsc::Sender<NetworkEvent>) {
    println!("Demo mode: injecting sample OT security events every 8s");
    println!("(Ctrl+C to stop)");

    let mut index = 0usize;
    loop {
        let event = match index % 3 {
            0 => demo_modbus_read(),
            1 => demo_modbus_unauthorized_write(),
            _ => demo_opcua(),
        };
        handle_event(event, &tx);
        index += 1;
        tokio::time::sleep(std::time::Duration::from_secs(8)).await;
    }
}

fn demo_modbus_read() -> NetworkEvent {
    NetworkEvent {
        timestamp: Utc::now(),
        source_ip: "192.168.10.30".to_string(),
        destination_ip: "192.168.10.10".to_string(),
        source_port: 55100,
        destination_port: 502,
        transport_protocol: "TCP".to_string(),
        application_protocol: Some("Modbus TCP".to_string()),
        payload_size: 12,
        modbus: Some(ModbusEvent {
            transaction_id: 1,
            unit_id: 1,
            function_code: 3,
            function_name: "Read Holding Registers".to_string(),
            is_write: false,
        }),
        opcua: None,
        suspicious: false,
        severity: None,
        message: None,
    }
}

fn demo_modbus_unauthorized_write() -> NetworkEvent {
    NetworkEvent {
        timestamp: Utc::now(),
        source_ip: "192.168.10.92".to_string(),
        destination_ip: "192.168.10.10".to_string(),
        source_port: 49812,
        destination_port: 502,
        transport_protocol: "TCP".to_string(),
        application_protocol: Some("Modbus TCP".to_string()),
        payload_size: 17,
        modbus: Some(ModbusEvent {
            transaction_id: 42,
            unit_id: 1,
            function_code: 16,
            function_name: "Write Multiple Registers".to_string(),
            is_write: true,
        }),
        opcua: None,
        suspicious: false,
        severity: None,
        message: None,
    }
}

fn demo_opcua() -> NetworkEvent {
    NetworkEvent {
        timestamp: Utc::now(),
        source_ip: "192.168.10.40".to_string(),
        destination_ip: "192.168.10.10".to_string(),
        source_port: 52010,
        destination_port: 4840,
        transport_protocol: "TCP".to_string(),
        application_protocol: Some("OPC UA".to_string()),
        payload_size: 64,
        modbus: None,
        opcua: Some(OpcUaEvent {
            message_type: "HEL".to_string(),
        }),
        suspicious: false,
        severity: None,
        message: None,
    }
}
