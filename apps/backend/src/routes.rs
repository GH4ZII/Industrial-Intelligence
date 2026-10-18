use axum::{
    extract::{Path, State},
    http::{header::CONTENT_TYPE, StatusCode},
    response::IntoResponse,
    Json,
};

use std::collections::BTreeMap;

use crate::{
    models::{
        AlertRow,
        AnomalyScoreRow,
        AssetDetail,
        AssetHealthRow,
        AssetRow,
        AssetSummary,
        DashboardSummary,
        HealthResponse,
        IncidentDetail,
        IncidentEventRow,
        IncidentRow,
        NetworkEventRow,
        PatchIncidentRequest,
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

    let health_by_asset = latest_health_by_asset(&state.pool).await?;
    let score_by_asset = latest_anomaly_by_asset(&state.pool).await?;

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
            asset_id: row.asset_id.clone(),
            status,
            health_percent: health_by_asset.get(&row.asset_id).copied(),
            anomaly_score: score_by_asset.get(&row.asset_id).copied(),
        });
    }

    Ok(assets)
}

async fn latest_health_by_asset(
    pool: &sqlx::PgPool,
) -> Result<BTreeMap<String, f64>, (StatusCode, String)> {
    let rows: Vec<(String, f64)> = sqlx::query_as(
        r#"
        SELECT DISTINCT ON (asset_id)
            asset_id,
            health_percent
        FROM asset_health
        ORDER BY asset_id, time DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Ok(rows.into_iter().collect())
}

async fn latest_anomaly_by_asset(
    pool: &sqlx::PgPool,
) -> Result<BTreeMap<String, f64>, (StatusCode, String)> {
    let rows: Vec<(String, f64)> = sqlx::query_as(
        r#"
        SELECT DISTINCT ON (asset_id)
            asset_id,
            score
        FROM anomaly_scores
        ORDER BY asset_id, time DESC
        "#,
    )
    .fetch_all(pool)
    .await
    .unwrap_or_default();

    Ok(rows.into_iter().collect())
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
            updated_at,
            severity,
            status,
            title,
            asset_id,
            description,
            assigned_to,
            mitre_id,
            mitre_name
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


pub async fn get_incident(
    Path(incident_id): Path<i64>,
    State(state): State<AppState>,
) -> Result<Json<IncidentDetail>, (StatusCode, String)> {
    let incident = sqlx::query_as::<_, IncidentRow>(
        r#"
        SELECT
            id,
            created_at,
            updated_at,
            severity,
            status,
            title,
            asset_id,
            description,
            assigned_to,
            mitre_id,
            mitre_name
        FROM incidents
        WHERE id = $1
        "#,
    )
    .bind(incident_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or((
        StatusCode::NOT_FOUND,
        format!("Incident {incident_id} not found"),
    ))?;

    let events = sqlx::query_as::<_, IncidentEventRow>(
        r#"
        SELECT id, incident_id, time, message
        FROM incident_events
        WHERE incident_id = $1
        ORDER BY time ASC, id ASC
        "#,
    )
    .bind(incident_id)
    .fetch_all(&state.pool)
    .await
    .map_err(internal_error)?;

    Ok(Json(IncidentDetail { incident, events }))
}

pub async fn patch_incident(
    Path(incident_id): Path<i64>,
    State(state): State<AppState>,
    Json(request): Json<PatchIncidentRequest>,
) -> Result<Json<IncidentRow>, (StatusCode, String)> {
    let current = sqlx::query_as::<_, IncidentRow>(
        r#"
        SELECT
            id,
            created_at,
            updated_at,
            severity,
            status,
            title,
            asset_id,
            description,
            assigned_to,
            mitre_id,
            mitre_name
        FROM incidents
        WHERE id = $1
        "#,
    )
    .bind(incident_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?
    .ok_or((
        StatusCode::NOT_FOUND,
        format!("Incident {incident_id} not found"),
    ))?;

    let status = request.status.unwrap_or(current.status.clone());
    match status.as_str() {
        "open" | "investigating" | "resolved" => {}
        other => {
            return Err((
                StatusCode::BAD_REQUEST,
                format!("Invalid status '{other}'. Use open, investigating, or resolved."),
            ));
        }
    }

    let assigned_to = request.assigned_to.or(current.assigned_to.clone());

    sqlx::query(
        r#"
        UPDATE incidents
        SET status = $2,
            assigned_to = $3,
            updated_at = NOW()
        WHERE id = $1
        "#,
    )
    .bind(incident_id)
    .bind(&status)
    .bind(&assigned_to)
    .execute(&state.pool)
    .await
    .map_err(internal_error)?;

    if status != current.status {
        sqlx::query(
            r#"
            INSERT INTO incident_events (incident_id, time, message)
            VALUES ($1, NOW(), $2)
            "#,
        )
        .bind(incident_id)
        .bind(format!("Status changed to {status}"))
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
    }

    if assigned_to != current.assigned_to {
        let label = assigned_to
            .as_deref()
            .unwrap_or("unassigned");
        sqlx::query(
            r#"
            INSERT INTO incident_events (incident_id, time, message)
            VALUES ($1, NOW(), $2)
            "#,
        )
        .bind(incident_id)
        .bind(format!("Assigned to {label}"))
        .execute(&state.pool)
        .await
        .map_err(internal_error)?;
    }

    get_incident_row(&state.pool, incident_id).await.map(Json)
}

async fn get_incident_row(
    pool: &sqlx::PgPool,
    incident_id: i64,
) -> Result<IncidentRow, (StatusCode, String)> {
    sqlx::query_as::<_, IncidentRow>(
        r#"
        SELECT
            id,
            created_at,
            updated_at,
            severity,
            status,
            title,
            asset_id,
            description,
            assigned_to,
            mitre_id,
            mitre_name
        FROM incidents
        WHERE id = $1
        "#,
    )
    .bind(incident_id)
    .fetch_one(pool)
    .await
    .map_err(internal_error)
}

pub async fn get_asset_anomalies(
    Path(asset_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<Vec<AnomalyScoreRow>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, AnomalyScoreRow>(
        r#"
        SELECT
            time,
            site,
            asset_id,
            score,
            is_anomaly,
            contributing_metrics,
            message
        FROM anomaly_scores
        WHERE asset_id = $1
        ORDER BY time DESC
        LIMIT 100
        "#,
    )
    .bind(asset_id)
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    Ok(Json(rows))
}

pub async fn get_asset_health(
    Path(asset_id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<AssetHealthRow>, (StatusCode, String)> {
    let row = sqlx::query_as::<_, AssetHealthRow>(
        r#"
        SELECT
            time,
            site,
            asset_id,
            health_percent,
            risk_7d,
            risk_30d,
            risk_90d,
            recommended_action
        FROM asset_health
        WHERE asset_id = $1
        ORDER BY time DESC
        LIMIT 1
        "#,
    )
    .bind(&asset_id)
    .fetch_optional(&state.pool)
    .await
    .map_err(internal_error)?;

    row.map(Json).ok_or((
        StatusCode::NOT_FOUND,
        format!("No health data for {asset_id} yet"),
    ))
}

pub async fn get_health_overview(
    State(state): State<AppState>,
) -> Result<Json<Vec<AssetHealthRow>>, (StatusCode, String)> {
    let rows = sqlx::query_as::<_, AssetHealthRow>(
        r#"
        SELECT DISTINCT ON (asset_id)
            time,
            site,
            asset_id,
            health_percent,
            risk_7d,
            risk_30d,
            risk_90d,
            recommended_action
        FROM asset_health
        ORDER BY asset_id, time DESC
        "#,
    )
    .fetch_all(&state.pool)
    .await
    .unwrap_or_default();

    Ok(Json(rows))
}

pub async fn metrics(State(state): State<AppState>) -> impl IntoResponse {
    let active_alerts: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM alerts WHERE acknowledged = false",
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let open_incidents: i64 = sqlx::query_scalar(
        "SELECT COUNT(*)::BIGINT FROM incidents WHERE status <> 'resolved'",
    )
    .fetch_one(&state.pool)
    .await
    .unwrap_or(0);

    let assets = load_assets_with_status(&state)
        .await
        .unwrap_or_default();
    let healthy = assets
        .iter()
        .filter(|asset| asset.status == "NORMAL")
        .count();
    let plant_health = if assets.is_empty() {
        100
    } else {
        ((healthy as f64 / assets.len() as f64) * 100.0).round() as i64
    };

    let body = format!(
        "# HELP industrial_active_alerts Unacknowledged alerts\n\
         # TYPE industrial_active_alerts gauge\n\
         industrial_active_alerts {active_alerts}\n\
         # HELP industrial_open_incidents Incidents not resolved\n\
         # TYPE industrial_open_incidents gauge\n\
         industrial_open_incidents {open_incidents}\n\
         # HELP industrial_plant_health_percent Share of assets in NORMAL status\n\
         # TYPE industrial_plant_health_percent gauge\n\
         industrial_plant_health_percent {plant_health}\n\
         # HELP industrial_assets_total Known assets with telemetry\n\
         # TYPE industrial_assets_total gauge\n\
         industrial_assets_total {}\n",
        assets.len()
    );

    ([(CONTENT_TYPE, "text/plain; version=0.0.4")], body)
}

fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {}", error),
    )
}