use std::collections::HashMap;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde_json::{json, Value};

/// Application error type. Variants map 1:1 to documented API error codes
/// (see `docs/milestones/01_auth_and_accounts.md` §4).
///
/// Response envelope: `{ "error": "<code>", "message": "<human>", "fields": {} }`.
#[derive(Debug, thiserror::Error)]
pub enum AppError {
    /// Request body or query failed validation. Per-field messages keyed by field name.
    #[error("validation failed")]
    Validation(HashMap<String, String>),

    /// Login: email/password mismatch. Generic to avoid email enumeration.
    #[error("invalid credentials")]
    InvalidCredentials,

    /// Sign-out-all / change-password: current password failed verification.
    #[error("invalid password")]
    InvalidPassword,

    /// Login: account exists but email isn't verified yet.
    #[error("email not verified")]
    EmailNotVerified,

    /// Registration / profile email change: address already in use.
    #[error("email already in use")]
    EmailInUse,

    /// Registration: captcha token rejected by the verifier.
    #[error("captcha verification failed")]
    CaptchaFailed,

    /// Verify-email / reset-password: code didn't match.
    #[error("invalid code")]
    InvalidCode,

    /// Verify-email / reset-password: code is past its expiry.
    #[error("code expired")]
    CodeExpired,

    /// Email-change verify-email called when no pending email change exists.
    #[error("no pending verification")]
    NoPendingVerification,

    /// Rate limit exceeded. `retry_after` seconds until the next allowed request.
    #[error("rate limited; retry after {retry_after}s")]
    RateLimited { retry_after: u64 },

    /// Missing or invalid auth cookie.
    #[error("unauthorized")]
    Unauthorized,

    /// Generic 404.
    #[error("not found")]
    NotFound,

    /// Database error. Logged at error level; never leaked to the client.
    #[error("database error: {0}")]
    Database(#[from] sqlx::Error),

    /// Catch-all for unexpected failures.
    #[error("internal error: {0}")]
    Internal(String),
}

impl AppError {
    /// Map a variant to its HTTP status, machine code, and user-facing message.
    fn parts(&self) -> (StatusCode, &'static str, String) {
        match self {
            AppError::Validation(_) => (
                StatusCode::BAD_REQUEST,
                "validation",
                "One or more fields are invalid.".into(),
            ),
            AppError::InvalidCredentials => (
                StatusCode::UNAUTHORIZED,
                "invalid_credentials",
                "Email or password is incorrect.".into(),
            ),
            AppError::InvalidPassword => (
                StatusCode::UNAUTHORIZED,
                "invalid_password",
                "Current password is incorrect.".into(),
            ),
            AppError::EmailNotVerified => (
                StatusCode::FORBIDDEN,
                "email_not_verified",
                "Verify your email before signing in.".into(),
            ),
            AppError::EmailInUse => (
                StatusCode::CONFLICT,
                "email_in_use",
                "That email is already registered.".into(),
            ),
            AppError::CaptchaFailed => (
                StatusCode::FORBIDDEN,
                "captcha_failed",
                "We couldn't verify the captcha. Try again.".into(),
            ),
            AppError::InvalidCode => (
                StatusCode::BAD_REQUEST,
                "invalid_code",
                "That code isn't right.".into(),
            ),
            AppError::CodeExpired => (
                StatusCode::BAD_REQUEST,
                "code_expired",
                "That code has expired. Request a new one.".into(),
            ),
            AppError::NoPendingVerification => (
                StatusCode::NOT_FOUND,
                "no_pending_verification",
                "No pending email change to verify.".into(),
            ),
            AppError::RateLimited { .. } => (
                StatusCode::TOO_MANY_REQUESTS,
                "rate_limited",
                "Too many requests. Please slow down.".into(),
            ),
            AppError::Unauthorized => (
                StatusCode::UNAUTHORIZED,
                "unauthorized",
                "Sign in to continue.".into(),
            ),
            AppError::NotFound => (
                StatusCode::NOT_FOUND,
                "not_found",
                "Not found.".into(),
            ),
            AppError::Database(_) | AppError::Internal(_) => (
                StatusCode::INTERNAL_SERVER_ERROR,
                "internal",
                "Something went wrong on our end.".into(),
            ),
        }
    }

    fn extras(&self) -> Value {
        match self {
            AppError::Validation(fields) => json!({ "fields": fields }),
            AppError::RateLimited { retry_after } => {
                json!({ "fields": {}, "retry_after_seconds": retry_after })
            }
            _ => json!({ "fields": {} }),
        }
    }
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, code, message) = self.parts();

        // Server-side faults get logged at error level; expected client-side outcomes are debug.
        match &self {
            AppError::Database(_) | AppError::Internal(_) => {
                tracing::error!(error = ?self, "request failed");
            }
            _ => tracing::debug!(error = ?self, "request rejected"),
        }

        let mut body = json!({
            "error": code,
            "message": message,
        });
        if let Value::Object(extras) = self.extras() {
            if let Value::Object(map) = &mut body {
                map.extend(extras);
            }
        }

        (status, Json(body)).into_response()
    }
}

impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        AppError::Internal(err.to_string())
    }
}

pub type AppResult<T> = Result<T, AppError>;
