use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkEvent {
    pub timestamp: DateTime<Utc>,
    pub source_ip: String,
    pub destination_ip: String,
    pub source_port: u16,
    pub destination_port: u16,
    pub transport_protocol: String,
    pub application_protocol: Option<String>,
    pub payload_size: usize,
    pub modbus: Option<ModbusEvent>,
    pub opcua: Option<OpcUaEvent>,
    pub suspicious: bool,
    pub severity: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModbusEvent {
    pub transaction_id: u16,
    pub unit_id: u8,
    pub function_code: u8,
    pub function_name: String,
    pub is_write: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OpcUaEvent {
    pub message_type: String,
}
