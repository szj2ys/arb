use axum::{
    extract::State,
    http::StatusCode,
    response::{Html, Json},
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
mod models;

use db::*;
use models::*;

#[derive(Clone)]
struct AppState {
    db: Pool<Sqlite>,
    admin_password: String,
}

#[derive(Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

#[derive(Serialize)]
struct SubmitResponse {
    id: String,
    message: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let database_url =
        std::env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:feedback.db".to_string());

    let admin_password = std::env::var("ADMIN_PASSWORD").unwrap_or_else(|| "admin123".to_string());

    let db = SqlitePoolOptions::new()
        .max_connections(5)
        .connect(&database_url)
        .await?;

    db::init(&db).await?;

    let state = Arc::new(AppState { db, admin_password });

    let app = Router::new()
        .route("/health", get(health_handler))
        .route("/v1/feedback", post(submit_feedback))
        .route("/v1/feedback/public", get(get_public_feedback))
        .route("/admin", get(admin_handler))
        .route("/admin/api/feedback", get(list_feedback))
        .route("/admin/api/feedback/:id/status", post(update_status))
        .layer(CorsLayer::permissive())
        .layer(TraceLayer::new_for_http())
        .with_state(state);

    let port = std::env::var("PORT")
        .unwrap_or_else(|_| "8081".to_string())
        .parse::<u16>()?;

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Feedback server listening on {}", addr);

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

async fn submit_feedback(
    State(state): State<Arc<AppState>>,
    Json(feedback): Json<FeedbackSubmit>,
) -> Result<Json<SubmitResponse>, StatusCode> {
    // Honeypot check
    if feedback.honeypot.is_some() && !feedback.honeypot.as_ref().unwrap().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }

    let id = uuid::Uuid::new_v4().to_string();

    match insert_feedback(&state.db, &id, &feedback).await {
        Ok(_) => Ok(Json(SubmitResponse {
            id,
            message: "Thank you for your feedback!".to_string(),
        })),
        Err(e) => {
            tracing::error!("Failed to insert feedback: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn get_public_feedback(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<PublicFeedback>>, StatusCode> {
    match get_public_feedback_list(&state.db).await {
        Ok(feedback) => Ok(Json(feedback)),
        Err(e) => {
            tracing::error!("Failed to get public feedback: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn admin_handler() -> Html<&'static str> {
    Html(include_str!("../static/admin.html"))
}

async fn list_feedback(
    State(state): State<Arc<AppState>>,
) -> Result<Json<Vec<Feedback>>, StatusCode> {
    match get_all_feedback(&state.db).await {
        Ok(feedback) => Ok(Json(feedback)),
        Err(e) => {
            tracing::error!("Failed to list feedback: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

async fn update_status(
    State(state): State<Arc<AppState>>,
    axum::extract::Path(id): axum::extract::Path<String>,
    Json(update): Json<StatusUpdate>,
) -> Result<StatusCode, StatusCode> {
    match update_feedback_status(&state.db, &id, &update.status).await {
        Ok(_) => Ok(StatusCode::OK),
        Err(e) => {
            tracing::error!("Failed to update status: {:?}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}
