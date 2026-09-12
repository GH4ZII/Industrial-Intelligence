use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
};

use std::collections::BTreeMap;

use crate::{
    models::{
        AlertRow,
        AssetDetail,
        AssetRow,
        AssetSummary,
        DashboardSummary,
        HealthResponse,
        IncidentRow,
        NetworkEventRow,
        SiteSummary,
        TelemetryRow,
    },
    AppState,
};

fn status_from_metrics(latest: &BTreeMap<String, f64>) -> String {
    let mut status = "NORMAL";

    if let Some(&temperature) = latest.get("temperature") {
        if temperature > 110.0 {
            return "CRITICAL".to_string();
        }
        if temperature > 90.0 {
            status = "WARNING";
        }
    }

    if let Some(&vibration) = latest.get("vibration") {
        if vibration > 9.0 {
            return "CRITICAL".to_string();
        }
        if vibration > 5.0 {
            status = "WARNING";
        }
    }

    if let Some(&pressure) = latest.get("pressure") {
        if pressure < 2.5 {
            return "CRITICAL".to_string();
        }
        if pressure < 3.5 {
            status = "WARNING";
        }
    }

    status.to_string()
}

fn worse_status(current: &str, candidate: &str) -> String {
    let rank = |s: &str| match s {
        "CRITICAL" => 2,
        "WARNING" => 1,
        _ => 0,
    };

    if rank(candidate) > rank(current) {
        candidate.to_string()
    } else {
        current.to_string()
    }
}

async fn status_from_alerts(
    pool: &sqlx::PgPool,
    asset_id: &str,
) -> Result<Option<String>, (StatusCode, String)> {
    let severities: Vec<(String,)> = sqlx::query_as(
        r#"
        SELECT severity
        FROM alerts
        WHERE asset_id = $1
          AND acknowledged = false
        "#,
    )
    .bind(asset_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    if severities.is_empty() {
        return Ok(None);
    }

    let mut status = "NORMAL".to_string();
    for (severity,) in severities {
        status = worse_status(&status, &severity);
    }

    Ok(Some(status))
}

async fn latest_metrics_for_asset(
    pool: &sqlx::PgPool,
    asset_id: &str,
) -> Result<BTreeMap<String, f64>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, TelemetryRow>(
        r#"
        SELECT DISTINCT ON (metric)
            time,
            site,
            asset_type,
            asset_id,
            metric,
            value
        FROM telemetry
        WHERE asset_id = $1
        ORDER BY metric, time DESC
        "#,
    )
    .bind(asset_id)
    .fetch_all(pool)
    .await
    .map_err(internal_error)?;

    let mut latest = BTreeMap::new();
    for row in rows {
        latest.insert(row.metric, row.value);
    }

    Ok(latest)
}


pub async fn health() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
    })
}


pub async fn get_sites(
    State(state): State<AppState>,
) -> Result<Json<Vec<SiteSummary>>, (StatusCode, String)> {

    let sites = sqlx::query_as::<_, SiteSummary>(
        r#"
        SELECT
            site AS name,
            COUNT(DISTINCT asset_id)::BIGINT AS asset_count
        FROM telemetry
        GROUP BY site
        ORDER BY site
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(sites))
}


pub async fn get_assets(
    State(state): State<AppState>,
) -> Result<Json<Vec<AssetSummary>>, (StatusCode, String)> {
    Ok(Json(load_assets_with_status(&state).await?))
}

pub async fn get_dashboard(
    State(state): State<AppState>,
) -> Result<Json<DashboardSummary>, (StatusCode, String)> {
    let assets = load_assets_with_status(&state).await?;

    let healthy = assets
        .iter()
        .filter(|asset| asset.status == "NORMAL")
        .count();

    let plant_health_percent = if assets.is_empty() {
        100
    } else {
        ((healthy as f64 / assets.len() as f64) * 100.0).round() as u32
    };

    let active_alerts: (i64,) = sqlx::query_as(
        r#"
        SELECT COUNT(*)::BIGINT
        FROM alerts
        WHERE acknowledged = false
        "#,
    )
    .fetch_one(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(DashboardSummary {
        plant_health_percent,
        active_alerts: active_alerts.0,
        assets,
    }))
}

pub async fn get_alerts(
    State(state): State<AppState>,
) -> Result<Json<Vec<AlertRow>>, (StatusCode, String)> {
    let alerts = sqlx::query_as::<_, AlertRow>(
        r#"
        SELECT
            id,
            time,
            site,
            asset_id,
            severity,
            alert_type,
            message,
            acknowledged
        FROM alerts
        WHERE acknowledged = false
        ORDER BY time DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(alerts))
}

async fn load_assets_with_status(
    state: &AppState,
) -> Result<Vec<AssetSummary>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, AssetRow>(
        r#"
        SELECT DISTINCT
            site,
            asset_type,
            asset_id
        FROM telemetry
        ORDER BY asset_type, asset_id
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    let mut assets = Vec::with_capacity(rows.len());

    for row in rows {
        let latest = latest_metrics_for_asset(&state.pool, &row.asset_id).await?;
        let status = match status_from_alerts(&state.pool, &row.asset_id).await? {
            Some(from_alerts) => from_alerts,
            None => status_from_metrics(&latest),
        };

        assets.push(AssetSummary {
            site: row.site,
            asset_type: row.asset_type,
            asset_id: row.asset_id,
            status,
        });
    }

    Ok(assets)
}


pub async fn get_asset(
    Path(asset_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<AssetDetail>, (StatusCode, String)> {

    let rows = sqlx::query_as::<_, TelemetryRow>(
        r#"
        SELECT DISTINCT ON (metric)
            time,
            site,
            asset_type,
            asset_id,
            metric,
            value
        FROM telemetry
        WHERE asset_id = $1
        ORDER BY metric, time DESC
        "#,
    )
    .bind(&asset_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;


    if rows.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Asset {} not found", asset_id),
        ));
    }


    let site = rows[0].site.clone();
    let asset_type = rows[0].asset_type.clone();

    let mut latest = BTreeMap::new();

    for row in rows {
        latest.insert(row.metric, row.value);
    }


    Ok(Json(AssetDetail {
        site,
        asset_type,
        asset_id,
        latest,
    }))
}


pub async fn get_asset_telemetry(
    Path(asset_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<TelemetryRow>>, (StatusCode, String)> {

    let telemetry = sqlx::query_as::<_, TelemetryRow>(
        r#"
        SELECT
            time,
            site,
            asset_type,
            asset_id,
            metric,
            value
        FROM telemetry
        WHERE asset_id = $1
        ORDER BY time DESC
        LIMIT 100
        "#,
    )
    .bind(asset_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(telemetry))
}


pub async fn get_asset_alerts(
    Path(asset_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<AlertRow>>, (StatusCode, String)> {

    let alerts = sqlx::query_as::<_, AlertRow>(
        r#"
        SELECT
            id,
            time,
            site,
            asset_id,
            severity,
            alert_type,
            message,
            acknowledged
        FROM alerts
        WHERE asset_id = $1
        ORDER BY time DESC
        "#,
    )
    .bind(asset_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(alerts))
}


pub async fn get_incidents(
    State(state): State<AppState>,
) -> Result<Json<Vec<IncidentRow>>, (StatusCode, String)> {

    let incidents = sqlx::query_as::<_, IncidentRow>(
        r#"
        SELECT
            id,
            created_at,
            severity,
            status,
            title,
            asset_id,
            description
        FROM incidents
        ORDER BY created_at DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(incidents))
}

pub async fn get_network_events(
    State(state): State<AppState>,
) -> Result<Json<Vec<NetworkEventRow>>, (StatusCode, String)> {
    // Table may not exist yet if security-monitor has never run.
    let events = sqlx::query_as::<_, NetworkEventRow>(
        r#"
        SELECT
            id,
            time,
            source_ip,
            destination_ip,
            source_port,
            destination_port,
            transport_protocol,
            application_protocol,
            payload_size,
            modbus_function,
            opcua_message,
            suspicious,
            severity,
            message
        FROM network_events
        ORDER BY time DESC
        LIMIT 100
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(events))
}


fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {}", error),
    )
}