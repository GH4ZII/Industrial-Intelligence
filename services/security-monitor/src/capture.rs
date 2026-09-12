use anyhow::{Context, Result};
use pcap::{Capture, Device};
use tokio::sync::mpsc;

use crate::detect;
use crate::models::NetworkEvent;
use crate::parser;

pub fn list_interfaces() -> Result<()> {
    let devices = Device::list().context("Failed to list Npcap devices")?;

    for device in devices {
        println!("Interface: {}", device.name);
        println!("Description: {:?}", device.desc);
        println!();
    }

    Ok(())
}

/// Prefer an explicit name, otherwise loopback, otherwise first device.
pub fn resolve_interface(preferred: Option<&str>) -> Result<String> {
    if let Some(name) = preferred {
        return Ok(name.to_string());
    }

    let devices = Device::list().context("Failed to list Npcap devices")?;

    if let Some(loopback) = devices.iter().find(|d| {
        d.name.contains("Loopback")
            || d.desc
                .as_ref()
                .map(|s| s.to_lowercase().contains("loopback"))
                .unwrap_or(false)
    }) {
        return Ok(loopback.name.clone());
    }

    devices
        .into_iter()
        .next()
        .map(|d| d.name)
        .context("No capture devices found. Is Npcap installed?")
}

pub fn start_capture(interface: &str, tx: mpsc::Sender<NetworkEvent>) -> Result<()> {
    let mut cap = Capture::from_device(interface)?
        .promisc(true)
        .immediate_mode(true)
        .timeout(500)
        .open()
        .with_context(|| format!("Failed to open capture on {interface}"))?;

    let _ = cap.filter("tcp port 502 or tcp port 4840 or tcp", true);

    println!("Listening on: {interface}");
    println!("Filter: tcp (highlight Modbus :502 / OPC UA :4840)");

    loop {
        match cap.next_packet() {
            Ok(packet) => {
                if let Some(event) = parser::parse_packet(packet.data) {
                    detect::handle_event(event, &tx);
                }
            }
            Err(pcap::Error::TimeoutExpired) => continue,
            Err(error) => {
                eprintln!("Capture error: {error}");
                break;
            }
        }
    }

    Ok(())
}
