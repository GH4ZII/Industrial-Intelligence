use crate::models::ModbusEvent;

/// Parse Modbus TCP ADU (MBAP header + PDU).
pub fn parse(payload: &[u8]) -> Option<ModbusEvent> {
    if payload.len() < 8 {
        return None;
    }

    let transaction_id = u16::from_be_bytes([payload[0], payload[1]]);
    let protocol_id = u16::from_be_bytes([payload[2], payload[3]]);
    if protocol_id != 0 {
        return None;
    }

    let length = u16::from_be_bytes([payload[4], payload[5]]) as usize;
    // length covers unit id + PDU
    if length < 2 || payload.len() < 6 + length {
        return None;
    }

    let unit_id = payload[6];
    let function_code = payload[7];
    let function_name = function_name(function_code).to_string();
    let is_write = is_write_function(function_code);

    Some(ModbusEvent {
        transaction_id,
        unit_id,
        function_code,
        function_name,
        is_write,
    })
}

fn function_name(code: u8) -> &'static str {
    match code {
        1 => "Read Coils",
        2 => "Read Discrete Inputs",
        3 => "Read Holding Registers",
        4 => "Read Input Registers",
        5 => "Write Single Coil",
        6 => "Write Single Register",
        15 => "Write Multiple Coils",
        16 => "Write Multiple Registers",
        22 => "Mask Write Register",
        23 => "Read/Write Multiple Registers",
        other if other >= 0x80 => "Exception Response",
        _ => "Unknown",
    }
}

fn is_write_function(code: u8) -> bool {
    matches!(code, 5 | 6 | 15 | 16 | 22 | 23)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_write_multiple_registers() {
        // MBAP + unit + fc16 + minimal data
        let payload = [
            0x00, 0x01, // transaction
            0x00, 0x00, // protocol
            0x00, 0x07, // length
            0x01, // unit
            0x10, // write multiple registers
            0x00, 0x00, 0x00, 0x01, 0x02, 0x00, 0x64,
        ];

        let event = parse(&payload).expect("parse");
        assert_eq!(event.function_code, 16);
        assert!(event.is_write);
        assert_eq!(event.function_name, "Write Multiple Registers");
    }
}
