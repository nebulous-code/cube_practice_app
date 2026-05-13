mod auth;
mod cases;
mod progress;
mod study;

use axum::{routing::get, Json, Router};
use serde_json::json;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .merge(auth::router())
        .merge(cases::router())
        .merge(progress::router())
        .merge(study::router())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
