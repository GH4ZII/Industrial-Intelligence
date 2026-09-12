use crate::models::OpcUaEvent;

/// Lightweight OPC UA binary Hello/Ack/OpenSecureChannel detection.
/// Full OPC UA decoding is out of scope for Phase 6; we identify activity on the wire.
pub fn parse(payload: &[u8]) -> Option<OpcUaEvent> {
    if payload.len() < 8 {
        return None;
    }

    // OPC UA TCP message header: MessageType (3 bytes) + ChunkType (1) + MessageSize (4)
    let message_type = std::str::from_utf8(&payload[0..3]).ok()?;

    let known = matches!(
        message_type,
        "HEL" | "ACK" | "ERR" | "OPN" | "MSG" | "CLO"
    );

    if !known {
        // Still mark as OPC UA by port even if header is incomplete/encrypted chunk
        return Some(OpcUaEvent {
            message_type: "Binary/Unknown".to_string(),
        });
    }

    Some(OpcUaEvent {
        message_type: message_type.to_string(),
    })
}
