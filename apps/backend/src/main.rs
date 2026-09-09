mod auth;
mod models;
mod routes;

use axum::{
    routing::{get, post},
    Router,
};

use sqlx::{
    postgres::PgPoolOptions,
    PgPool,
};


#[derive(Clone)]
pub struct AppState {
    pub pool: PgPool,
}

use tower_http::cors::{
    Any,
    CorsLayer,
};


#[tokio::main]
async fn main() {

    println!(
        "Starting Industrial Intelligence Backend..."
    );


    let database_url =
        "postgres://industrial:industrial123@localhost:5432/industrial_intelligence";


    let pool =
        PgPoolOptions::new()
            .max_connections(10)
            .connect(database_url)
            .await
            .expect(
                "Failed to connect to TimescaleDB"
            );


    println!("Connected to TimescaleDB.");


    let state = AppState {
        pool,
    };


    let app = Router::new()

        // Health
        .route(
            "/health",
            get(routes::health),
        )

        // Sites
        .route(
            "/api/sites",
            get(routes::get_sites),
        )

        // Dashboard
        .route(
            "/api/dashboard",
            get(routes::get_dashboard),
        )

        // Alerts
        .route(
            "/api/alerts",
            get(routes::get_alerts),
        )

        // Assets
        .route(
            "/api/assets",
            get(routes::get_assets),
        )

        .route(
            "/api/assets/{asset_id}",
            get(routes::get_asset),
        )

        .route(
            "/api/assets/{asset_id}/telemetry",
            get(routes::get_asset_telemetry),
        )

        .route(
            "/api/assets/{asset_id}/alerts",
            get(routes::get_asset_alerts),
        )

        // Incidents
        .route(
            "/api/incidents",
            get(routes::get_incidents),
        )

        // Authentication
        .route(
            "/api/auth/register",
            post(auth::register),
        )

        .route(
            "/api/auth/login",
            post(auth::login),
        )

        // Users
        .route(
            "/api/users/me",
            get(auth::me),
        )

        .route(
            "/api/users",
            get(auth::get_users),
        )

        .route(
            "/api/roles",
            get(auth::get_roles),
        )

        .with_state(state)

        .layer(CorsLayer::new()
            .allow_origin(Any)
            .allow_methods(Any)
            .allow_headers(Any)
        );


    let listener =
        tokio::net::TcpListener::bind(
            "0.0.0.0:3000"
        )
        .await
        .unwrap();


    println!(
        "Backend running on http://localhost:3000"
    );


    axum::serve(listener, app)
        .await
        .unwrap();
}