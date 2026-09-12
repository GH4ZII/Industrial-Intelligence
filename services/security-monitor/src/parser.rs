use chrono::Utc;
use etherparse::{NetSlice, SlicedPacket, TransportSlice};

use crate::models::NetworkEvent;
use crate::protocols::{modbus, opcua};

pub fn parse_packet(data: &[u8]) -> Option<NetworkEvent> {
    let sliced = SlicedPacket::from_ethernet(data).ok()?;

    let (source_ip, destination_ip) = match sliced.net.as_ref()? {
        NetSlice::Ipv4(ipv4) => {
            let header = ipv4.header();
            (
                header.source_addr().to_string(),
                header.destination_addr().to_string(),
            )
        }
        NetSlice::Ipv6(ipv6) => {
            let header = ipv6.header();
            (
                header.source_addr().to_string(),
                header.destination_addr().to_string(),
            )
        }
        // Ignore non-IP (ARP etc.) for Phase 6 industrial focus
        _ => return None,
    };

    let transport = sliced.transport.as_ref()?;

    let (source_port, destination_port, transport_protocol, payload) = match transport {
        TransportSlice::Tcp(tcp) => (
            tcp.source_port(),
            tcp.destination_port(),
            "TCP".to_string(),
            tcp.payload(),
        ),
        TransportSlice::Udp(udp) => (
            udp.source_port(),
            udp.destination_port(),
            "UDP".to_string(),
            udp.payload(),
        ),
        _ => return None,
    };

    let modbus_event = if destination_port == 502 || source_port == 502 {
        modbus::parse(payload)
    } else {
        None
    };

    let opcua_event = if destination_port == 4840 || source_port == 4840 {
        opcua::parse(payload)
    } else {
        None
    };

    let application_protocol = if modbus_event.is_some() {
        Some("Modbus TCP".to_string())
    } else if opcua_event.is_some() {
        Some("OPC UA".to_string())
    } else if destination_port == 502 || source_port == 502 {
        Some("Modbus TCP".to_string())
    } else if destination_port == 4840 || source_port == 4840 {
        Some("OPC UA".to_string())
    } else {
        None
    };

    // Only keep industrial-relevant or explicitly parsed traffic in the event stream.
    if application_protocol.is_none() {
        return None;
    }

    Some(NetworkEvent {
        timestamp: Utc::now(),
        source_ip,
        destination_ip,
        source_port,
        destination_port,
        transport_protocol,
        application_protocol,
        payload_size: payload.len(),
        modbus: modbus_event,
        opcua: opcua_event,
        suspicious: false,
        severity: None,
        message: None,
    })
}
