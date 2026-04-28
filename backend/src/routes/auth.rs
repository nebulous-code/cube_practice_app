//! Auth endpoints. See `docs/milestones/01_auth_and_accounts.md` §4.

use std::collections::HashMap;

use axum::{extract::State, routing::post, Json, Router};
use chrono::{Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{code::six_digit_code, password::hash_password};
use crate::email::{verification, ResendClient};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

const VERIFICATION_TTL_MINUTES: i64 = 10;

pub fn router() -> Router<AppState> {
    Router::new().route("/auth/register", post(register))
}

#[derive(Debug, Deserialize)]
pub struct RegisterRequest {
    display_name: String,
    email: String,
    password: String,
    /// reCAPTCHA v3 token from the frontend. Empty string in dev / curl testing
    /// is acceptable when `RECAPTCHA_SECRET_KEY` is unset on the backend.
    #[serde(default)]
    recaptcha_token: String,
}

#[derive(Debug, Serialize)]
pub struct RegisterResponse {
    id: Uuid,
    email: String,
    display_name: String,
    email_verified: bool,
}

async fn register(
    State(state): State<AppState>,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<RegisterResponse>> {
    let display_name = req.display_name.trim().to_string();
    let email = req.email.trim().to_lowercase();

    validate_register(&display_name, &email, &req.password)?;
    state.recaptcha.verify(&req.recaptcha_token).await?;

    let password_hash = hash_password(&req.password, state.argon2)?;
    let code = six_digit_code();
    let expires = Utc::now() + Duration::minutes(VERIFICATION_TTL_MINUTES);

    let row: Result<(Uuid, String, String, bool), sqlx::Error> = sqlx::query_as(
        r#"
        INSERT INTO users (email, display_name, password_hash, verification_code, verification_code_expires)
        VALUES ($1, $2, $3, $4, $5)
        RETURNING id, email, display_name, email_verified
        "#,
    )
    .bind(&email)
    .bind(&display_name)
    .bind(&password_hash)
    .bind(&code)
    .bind(expires)
    .fetch_one(&state.pool)
    .await;

    let (id, email, display_name, email_verified) = match row {
        Ok(r) => r,
        Err(e) if is_unique_violation(&e) => return Err(AppError::EmailInUse),
        Err(e) => return Err(e.into()),
    };

    send_verification_email(&state.email, &email, &code).await?;

    tracing::info!(user_id = %id, email = %email, "registered new user");

    Ok(Json(RegisterResponse {
        id,
        email,
        display_name,
        email_verified,
    }))
}

fn validate_register(display_name: &str, email: &str, password: &str) -> AppResult<()> {
    let mut fields: HashMap<String, String> = HashMap::new();

    if display_name.is_empty() {
        fields.insert("display_name".into(), "Required.".into());
    } else if display_name.chars().count() > 80 {
        fields.insert("display_name".into(), "Must be 80 characters or fewer.".into());
    }

    if !is_email_shape(email) {
        fields.insert("email".into(), "Enter a valid email address.".into());
    }

    if password.chars().count() < 8 {
        fields.insert("password".into(), "Must be at least 8 characters.".into());
    }

    if fields.is_empty() {
        Ok(())
    } else {
        Err(AppError::Validation(fields))
    }
}

/// Server-side email shape check — intentionally loose. We don't try to be
/// RFC-perfect; the only real validation is "did the verification email
/// reach you?". Stricter regexes mainly reject valid-but-uncommon addresses.
fn is_email_shape(s: &str) -> bool {
    let s = s.trim();
    let at_count = s.matches('@').count();
    if at_count != 1 || s.starts_with('@') || s.ends_with('@') || s.contains(' ') {
        return false;
    }
    let (local, domain) = s.split_once('@').unwrap();
    !local.is_empty() && domain.contains('.') && !domain.starts_with('.') && !domain.ends_with('.')
}

fn is_unique_violation(err: &sqlx::Error) -> bool {
    err.as_database_error()
        .and_then(|e| e.code())
        .map(|c| c == "23505")
        .unwrap_or(false)
}

async fn send_verification_email(email: &ResendClient, to: &str, code: &str) -> AppResult<()> {
    let msg = verification(code);
    email.send(to, &msg.subject, &msg.text, &msg.html).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_accepts_minimum() {
        assert!(validate_register("a", "x@y.z", "12345678").is_ok());
    }

    #[test]
    fn validate_collects_all_field_errors() {
        let err = validate_register("", "not-an-email", "short").unwrap_err();
        match err {
            AppError::Validation(fields) => {
                assert!(fields.contains_key("display_name"));
                assert!(fields.contains_key("email"));
                assert!(fields.contains_key("password"));
            }
            _ => panic!("expected Validation"),
        }
    }

    #[test]
    fn email_shape_accepts_normal_addresses() {
        assert!(is_email_shape("alice@example.com"));
        assert!(is_email_shape("a.b+tag@sub.example.co.uk"));
    }

    #[test]
    fn email_shape_rejects_obvious_garbage() {
        for bad in ["", "no-at", "@nope.com", "trailing@", "two@@signs.com", "spa ce@x.com", "noTLD@x"] {
            assert!(!is_email_shape(bad), "{bad} should be rejected");
        }
    }
}
