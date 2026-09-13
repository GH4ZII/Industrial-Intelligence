use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::BTreeMap;
use uuid::Uuid;


#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
}


#[derive(Debug, Serialize, FromRow)]
pub struct SiteSummary {
    pub name: String,
    pub asset_count: i64,
}


#[derive(Debug, Serialize, FromRow)]
pub struct AssetRow {
    pub site: String,
    pub asset_type: String,
    pub asset_id: String,
}

#[derive(Debug, Serialize)]
pub struct AssetSummary {
    pub site: String,
    pub asset_type: String,
    pub asset_id: String,
    pub status: String,
    pub health_percent: Option<f64>,
    pub anomaly_score: Option<f64>,
}

#[derive(Debug, Serialize)]
pub struct DashboardSummary {
    pub plant_health_percent: u32,
    pub active_alerts: i64,
    pub assets: Vec<AssetSummary>,
}


#[derive(Debug, Serialize, FromRow)]
pub struct TelemetryRow {
    pub time: DateTime<Utc>,
    pub site: String,
    pub asset_type: String,
    pub asset_id: String,
    pub metric: String,
    pub value: f64,
}


#[derive(Debug, Serialize)]
pub struct AssetDetail {
    pub site: String,
    pub asset_type: String,
    pub asset_id: String,
    pub latest: BTreeMap<String, f64>,
}


#[derive(Debug, Serialize, FromRow)]
pub struct AlertRow {
    pub id: i64,
    pub time: DateTime<Utc>,
    pub site: String,
    pub asset_id: String,
    pub severity: String,
    pub alert_type: String,
    pub message: String,
    pub acknowledged: bool,
}


#[derive(Debug, Serialize, FromRow)]
pub struct IncidentRow {
    pub id: i64,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub severity: String,
    pub status: String,
    pub title: String,
    pub asset_id: Option<String>,
    pub description: Option<String>,
    pub assigned_to: Option<String>,
    pub mitre_id: Option<String>,
    pub mitre_name: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct IncidentEventRow {
    pub id: i64,
    pub incident_id: i64,
    pub time: DateTime<Utc>,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct IncidentDetail {
    pub incident: IncidentRow,
    pub events: Vec<IncidentEventRow>,
}

#[derive(Debug, Deserialize)]
pub struct PatchIncidentRequest {
    pub status: Option<String>,
    pub assigned_to: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AnomalyScoreRow {
    pub time: DateTime<Utc>,
    pub site: String,
    pub asset_id: String,
    pub score: f64,
    pub is_anomaly: bool,
    pub contributing_metrics: Option<String>,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct AssetHealthRow {
    pub time: DateTime<Utc>,
    pub site: String,
    pub asset_id: String,
    pub health_percent: f64,
    pub risk_7d: f64,
    pub risk_30d: f64,
    pub risk_90d: f64,
    pub recommended_action: Option<String>,
}

#[derive(Debug, Serialize, FromRow)]
pub struct NetworkEventRow {
    pub id: i64,
    pub time: DateTime<Utc>,
    pub source_ip: String,
    pub destination_ip: String,
    pub source_port: i32,
    pub destination_port: i32,
    pub transport_protocol: String,
    pub application_protocol: Option<String>,
    pub payload_size: i32,
    pub modbus_function: Option<String>,
    pub opcua_message: Option<String>,
    pub suspicious: bool,
    pub severity: Option<String>,
    pub message: Option<String>,
}


#[derive(Debug, Serialize, FromRow)]
pub struct UserPublic {
    pub id: i64,
    pub email: String,
    pub role: String,
}


#[derive(Debug, Serialize, FromRow)]
pub struct Role {
    pub id: i32,
    pub name: String,
}


#[derive(Deserialize)]
pub struct RegisterRequest {
    pub email: String,
    pub password: String,
}


#[derive(Deserialize)]
pub struct LoginRequest {
    pub email: String,
    pub password: String,
}


#[derive(Serialize)]
pub struct LoginResponse {
    pub token: Uuid,
    pub expires_at: DateTime<Utc>,
    pub user: UserPublic,
}


#[derive(FromRow)]
pub struct LoginUser {
    pub id: i64,
    pub email: String,
    pub password_hash: String,
    pub role: String,
}