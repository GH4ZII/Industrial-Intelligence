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
        AssetSummary,
        HealthResponse,
        IncidentRow,
        SiteSummary,
        TelemetryRow,
    },
    AppState,
};


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

    let assets = sqlx::query_as::<_, AssetSummary>(
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

    Ok(Json(assets))
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


fn internal_error(error: sqlx::Error) -> (StatusCode, String) {
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        format!("Database error: {}", error),
    )
}