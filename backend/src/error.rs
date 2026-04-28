use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::json;

/// Application error type. Variants map to specific status codes and error envelopes.
/// Envelope shape: `{ "error": "<machine_code>", "message": "<human-readable>", "fields": {} }`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error("internal error: {0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = match &self {
            AppError::Internal(_) => (StatusCode::INTERNAL_SERVER_ERROR, "internal", "Something went wrong."),
        };

        tracing::error!(error = ?self, "request failed");

        (
            status,
            Json(json!({
                "error": code,
                "message": message,
                "fields": {},
            })),
        )
            .into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

impl From<sqlx::Error> for AppError {
    fn from(err: sqlx::Error) -> Self {
        AppError::Internal(format!("database error: {err}"))
    }
}
