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
pub struct AssetSummary {
    pub site: String,
    pub asset_type: String,
    pub asset_id: String,
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
    pub severity: String,
    pub status: String,
    pub title: String,
    pub asset_id: Option<String>,
    pub description: Option<String>,
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