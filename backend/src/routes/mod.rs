mod auth;
mod cases;

use axum::{routing::get, Json, Router};
use serde_json::json;

use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/health", get(health))
        .merge(auth::router())
        .merge(cases::router())
}

async fn health() -> Json<serde_json::Value> {
    Json(json!({ "status": "ok" }))
}
