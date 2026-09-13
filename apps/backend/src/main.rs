mod auth;
mod models;
mod routes;
mod schema;

use std::time::Duration;

use axum::{
    routing::{get, post},
    Router,
};

use sqlx::{postgres::PgPoolOptions, PgPool};

use tower_http::cors::{Any, CorsLayer};

#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

async fn connect_db() -> PgPool {
    let database_url = schema::database_url();
    println!("Database: {database_url}");

    for attempt in 1..=30 {
        match PgPoolOptions::new()
            .max_connections(10)
            .connect(&database_url)
            .await
        {
            Ok(pool) => return pool,
            Err(error) => {
                eprintln!("TimescaleDB connect attempt {attempt}/30 failed: {error}");
                tokio::time::sleep(Duration::from_secs(2)).await;
            }
        }
    }

    panic!("Failed to connect to TimescaleDB");
}

#[tokio::main]
async fn main() {
    println!("Starting Industrial Intelligence Backend...");

    let pool = connect_db().await;
    schema::ensure_schema(&pool)
        .await
        .expect("Failed to apply schema");
    println!("Connected to TimescaleDB.");

    let state = AppState { pool };

    let app = Router::new()
        .route("/health", get(routes::health))
        .route("/metrics", get(routes::metrics))
        .route("/api/sites", get(routes::get_sites))
        .route("/api/dashboard", get(routes::get_dashboard))
        .route("/api/alerts", get(routes::get_alerts))
        .route("/api/health-overview", get(routes::get_health_overview))
        .route("/api/assets", get(routes::get_assets))
        .route("/api/assets/{asset_id}", get(routes::get_asset))
        .route(
            "/api/assets/{asset_id}/telemetry",
            get(routes::get_asset_telemetry),
        )
        .route(
            "/api/assets/{asset_id}/alerts",
            get(routes::get_asset_alerts),
        )
        .route(
            "/api/assets/{asset_id}/anomalies",
            get(routes::get_asset_anomalies),
        )
        .route(
            "/api/assets/{asset_id}/health",
            get(routes::get_asset_health),
        )
        .route("/api/incidents", get(routes::get_incidents))
        .route(
            "/api/incidents/{incident_id}",
            get(routes::get_incident).patch(routes::patch_incident),
        )
        .route("/api/security/events", get(routes::get_network_events))
        .route("/api/auth/register", post(auth::register))
        .route("/api/auth/login", post(auth::login))
        .route("/api/users/me", get(auth::me))
        .route("/api/users", get(auth::get_users))
        .route("/api/roles", get(auth::get_roles))
        .with_state(state)
        .layer(
            CorsLayer::new()
                .allow_origin(Any)
                .allow_methods(Any)
                .allow_headers(Any),
        );

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("Backend running on http://localhost:3000");

    axum::serve(listener, app).await.unwrap();
}
