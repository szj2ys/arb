use axum::{
    extract::State,
    http::StatusCode,
    response::Json,
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqlitePoolOptions, Pool, Sqlite};
use std::net::SocketAddr;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::trace::TraceLayer;
use tracing;

mod db;

use db::*;

#[derive(Clone)]
struct AppState {
    db: Pool<Sqlite>,
}

#[derive(Debug, Deserialize)]
struct EventBatch {
    events: Vec<TelemetryEvent>,
}

#[derive(Debug, Deserialize)]
struct TelemetryEvent {
    device_id_hash: String,
    timestamp: i64,
    #[serde(flatten)]
    event_type: EventType,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
enum EventType {
    Install { method: String },
    FirstLaunch { version: String },
    ShellInit { shell: String },
    FeatureUse { feature: String },
    UpdateCheck { has_update: bool },
    Feedback { category: String },
    Diagnostic { issues_found: u32 },
}

#[derive(Serialize)]
struct SubmitResponse {
    received: usize,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "sqlite:telemetry.db".to_string());

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    db::init(&db).await?;

    let state = Arc::new(AppState { db });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/v1/events", post(submit_events))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8080".to_string())
        .parse::<u16>()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Telemetry server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn submit_events(
    State(state): State<Arc<AppState>>,
    Json(batch): Json<EventBatch>,
) -> Result<Json<SubmitResponse>, StatusCode> {
    let count = batch.events.len();

    for event in batch.events {
        if let Err(e) = insert_event(&state.db, &event).await {
            tracing::error!("Failed to insert event: {:?}", e);
        }
    }

    Ok(Json(SubmitResponse { received: count }))
}
