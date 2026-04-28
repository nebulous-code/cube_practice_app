//! Auth endpoints. See `docs/milestones/01_auth_and_accounts.md` §4.

use std::collections::HashMap;

use axum::{extract::State, routing::post, Json, Router};
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{
    code::six_digit_code,
    cookie::clear_session_cookie,
    extractor::AuthUser,
    password::{hash_password, verify_password},
    session::create_session,
};
use crate::email::{email_change_verification, verification, ResendClient};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

const VERIFICATION_TTL_MINUTES: i64 = 10;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/verify-email", post(verify_email))
        .route("/auth/resend-verification", post(resend_verification))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/me", axum::routing::get(me))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/register
// ─────────────────────────────────────────────────────────────────────────────

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

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/verify-email
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct VerifyEmailRequest {
    code: String,
    /// Required for the unauthenticated (initial registration) flow.
    /// Ignored when the request carries a valid session.
    #[serde(default)]
    email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct VerifyEmailResponse {
    id: Uuid,
    email: String,
    display_name: String,
    email_verified: bool,
}

async fn verify_email(
    State(state): State<AppState>,
    jar: CookieJar,
    auth_user: Option<AuthUser>,
    Json(req): Json<VerifyEmailRequest>,
) -> AppResult<(CookieJar, Json<VerifyEmailResponse>)> {
    let code = req.code.trim().to_string();
    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        return Err(AppError::InvalidCode);
    }

    if let Some(user) = auth_user {
        // Authenticated mode: promote pending_email.
        let resp = verify_email_change(&state, user.user_id, &code).await?;
        Ok((jar, Json(resp)))
    } else {
        // Unauthenticated mode: initial verification, also signs the user in.
        let email = req
            .email
            .as_deref()
            .map(|e| e.trim().to_lowercase())
            .filter(|e| !e.is_empty())
            .ok_or_else(|| {
                let mut fields = HashMap::new();
                fields.insert("email".into(), "Required.".into());
                AppError::Validation(fields)
            })?;

        let resp = verify_initial(&state, &email, &code).await?;
        let session = create_session(&state.pool, resp.id, &state.config.jwt_secret).await?;
        Ok((jar.add(session.cookie), Json(resp)))
    }
}

/// Initial verification. Returns user + sets `email_verified=true`. Caller is
/// expected to attach a session cookie on success.
async fn verify_initial(
    state: &AppState,
    email: &str,
    code: &str,
) -> AppResult<VerifyEmailResponse> {
    let row: Option<(Uuid, String, String, Option<String>, Option<DateTime<Utc>>, bool)> =
        sqlx::query_as(
            r#"
            SELECT id, email, display_name, verification_code, verification_code_expires, email_verified
            FROM users
            WHERE email = $1
            "#,
        )
        .bind(email)
        .fetch_optional(&state.pool)
        .await?;

    let (id, db_email, display_name, stored_code, expires, already_verified) =
        row.ok_or(AppError::InvalidCode)?;

    // Already-verified users: idempotent success on the right code; reject otherwise.
    if already_verified {
        return match stored_code {
            Some(stored) if stored == code => Ok(VerifyEmailResponse {
                id,
                email: db_email,
                display_name,
                email_verified: true,
            }),
            _ => Err(AppError::InvalidCode),
        };
    }

    let stored = stored_code.ok_or(AppError::InvalidCode)?;
    let expires = expires.ok_or(AppError::InvalidCode)?;
    if stored != code {
        return Err(AppError::InvalidCode);
    }
    if expires <= Utc::now() {
        return Err(AppError::CodeExpired);
    }

    sqlx::query(
        r#"
        UPDATE users
        SET email_verified = TRUE,
            verification_code = NULL,
            verification_code_expires = NULL
        WHERE id = $1
        "#,
    )
    .bind(id)
    .execute(&state.pool)
    .await?;

    tracing::info!(user_id = %id, "verified email — initial registration");

    Ok(VerifyEmailResponse {
        id,
        email: db_email,
        display_name,
        email_verified: true,
    })
}

/// Email-change verification. Promotes `pending_email` into `email`.
async fn verify_email_change(
    state: &AppState,
    user_id: Uuid,
    code: &str,
) -> AppResult<VerifyEmailResponse> {
    let row: Option<(Uuid, String, String, Option<String>, Option<String>, Option<DateTime<Utc>>, bool)> =
        sqlx::query_as(
            r#"
            SELECT id, email, display_name, pending_email, verification_code, verification_code_expires, email_verified
            FROM users
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .fetch_optional(&state.pool)
        .await?;

    let (id, _email, display_name, pending, stored_code, expires, email_verified) =
        row.ok_or(AppError::Unauthorized)?;
    let pending = pending.ok_or(AppError::NoPendingVerification)?;
    let stored = stored_code.ok_or(AppError::InvalidCode)?;
    let expires = expires.ok_or(AppError::InvalidCode)?;
    if stored != code {
        return Err(AppError::InvalidCode);
    }
    if expires <= Utc::now() {
        return Err(AppError::CodeExpired);
    }

    let result: Result<(), sqlx::Error> = sqlx::query(
        r#"
        UPDATE users
        SET email = $2,
            pending_email = NULL,
            verification_code = NULL,
            verification_code_expires = NULL,
            email_verified = TRUE
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(&pending)
    .execute(&state.pool)
    .await
    .map(|_| ());

    match result {
        Ok(()) => Ok(VerifyEmailResponse {
            id,
            email: pending,
            display_name,
            email_verified: true,
        }),
        // Race: someone else registered the pending address first.
        Err(e) if is_unique_violation(&e) => Err(AppError::EmailInUse),
        Err(e) => Err(e.into()),
    }
    .map(|resp| {
        let _ = email_verified; // currently unused; will be relevant when email-change banner state surfaces
        resp
    })
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/resend-verification
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ResendVerificationRequest {
    /// Required for the unauthenticated path. Ignored when authenticated.
    #[serde(default)]
    email: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct EmptyResponse {}

async fn resend_verification(
    State(state): State<AppState>,
    auth_user: Option<AuthUser>,
    Json(req): Json<ResendVerificationRequest>,
) -> AppResult<Json<EmptyResponse>> {
    // TODO: rate-limit 1/min per user once B9 lands.

    if let Some(user) = auth_user {
        resend_for_user(&state, user.user_id).await?;
    } else {
        let email = req
            .email
            .as_deref()
            .map(|e| e.trim().to_lowercase())
            .filter(|e| !e.is_empty())
            .ok_or_else(|| {
                let mut fields = HashMap::new();
                fields.insert("email".into(), "Required.".into());
                AppError::Validation(fields)
            })?;
        resend_for_email(&state, &email).await?;
    }

    Ok(Json(EmptyResponse {}))
}

async fn resend_for_email(state: &AppState, email: &str) -> AppResult<()> {
    let row: Option<(Uuid, Option<String>, bool)> = sqlx::query_as(
        "SELECT id, pending_email, email_verified FROM users WHERE email = $1",
    )
    .bind(email)
    .fetch_optional(&state.pool)
    .await?;

    let Some((id, _pending, verified)) = row else {
        // Don't reveal whether the email is registered.
        return Ok(());
    };
    if verified {
        // Already verified initial registration; nothing to resend at the unauth path.
        return Ok(());
    }

    let code = issue_new_code(&state.pool, id).await?;
    send_verification_email(&state.email, email, &code).await
}

async fn resend_for_user(state: &AppState, user_id: Uuid) -> AppResult<()> {
    let row: Option<(String, Option<String>, bool)> = sqlx::query_as(
        "SELECT email, pending_email, email_verified FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&state.pool)
    .await?;

    let (email, pending, verified) = row.ok_or(AppError::Unauthorized)?;
    let code = issue_new_code(&state.pool, user_id).await?;

    if let Some(pending_email) = pending {
        // Email-change flow: send to the *new* (pending) address.
        let msg = email_change_verification(&code, &pending_email);
        state
            .email
            .send(&pending_email, &msg.subject, &msg.text, &msg.html)
            .await
    } else if !verified {
        // Initial registration but the user is already authed (rare but possible during edge flows).
        send_verification_email(&state.email, &email, &code).await
    } else {
        // Authed user, no pending change, already verified — nothing to do.
        Ok(())
    }
}

async fn issue_new_code(pool: &sqlx::PgPool, user_id: Uuid) -> AppResult<String> {
    let code = six_digit_code();
    let expires = Utc::now() + Duration::minutes(VERIFICATION_TTL_MINUTES);
    sqlx::query(
        r#"
        UPDATE users
        SET verification_code = $2, verification_code_expires = $3
        WHERE id = $1
        "#,
    )
    .bind(user_id)
    .bind(&code)
    .bind(expires)
    .execute(pool)
    .await?;
    Ok(code)
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/login
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct LoginRequest {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct LoginResponse {
    id: Uuid,
    email: String,
    display_name: String,
}

async fn login(
    State(state): State<AppState>,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> AppResult<(CookieJar, Json<LoginResponse>)> {
    let email = req.email.trim().to_lowercase();

    let row: Option<(Uuid, String, String, String, bool)> = sqlx::query_as(
        "SELECT id, email, display_name, password_hash, email_verified FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await?;

    let (id, email, display_name, password_hash, email_verified) =
        row.ok_or(AppError::InvalidCredentials)?;

    if !verify_password(&req.password, &password_hash)? {
        return Err(AppError::InvalidCredentials);
    }
    if !email_verified {
        return Err(AppError::EmailNotVerified);
    }

    let session = create_session(&state.pool, id, &state.config.jwt_secret).await?;

    tracing::info!(user_id = %id, "user signed in");

    Ok((
        jar.add(session.cookie),
        Json(LoginResponse {
            id,
            email,
            display_name,
        }),
    ))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/logout
// ─────────────────────────────────────────────────────────────────────────────

async fn logout(
    State(state): State<AppState>,
    jar: CookieJar,
    user: AuthUser,
) -> AppResult<(CookieJar, Json<EmptyResponse>)> {
    sqlx::query("UPDATE sessions SET revoked = TRUE WHERE id = $1")
        .bind(user.session_id)
        .execute(&state.pool)
        .await?;

    Ok((jar.add(clear_session_cookie()), Json(EmptyResponse {})))
}

// ─────────────────────────────────────────────────────────────────────────────
// GET /auth/me
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct MeResponse {
    id: Uuid,
    email: String,
    pending_email: Option<String>,
    display_name: String,
    email_verified: bool,
}

async fn me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<MeResponse>> {
    let row: Option<(Uuid, String, Option<String>, String, bool)> = sqlx::query_as(
        "SELECT id, email, pending_email, display_name, email_verified FROM users WHERE id = $1",
    )
    .bind(user.user_id)
    .fetch_optional(&state.pool)
    .await?;

    let (id, email, pending_email, display_name, email_verified) =
        row.ok_or(AppError::Unauthorized)?;

    Ok(Json(MeResponse {
        id,
        email,
        pending_email,
        display_name,
        email_verified,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

fn validate_register(display_name: &str, email: &str, password: &str) -> AppResult<()> {
    let mut fields: HashMap<String, String> = HashMap::new();

    if display_name.is_empty() {
        fields.insert("display_name".into(), "Required.".into());
    } else if display_name.chars().count() > 80 {
        fields.insert(
            "display_name".into(),
            "Must be 80 characters or fewer.".into(),
        );
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
        for bad in [
            "",
            "no-at",
            "@nope.com",
            "trailing@",
            "two@@signs.com",
            "spa ce@x.com",
            "noTLD@x",
        ] {
            assert!(!is_email_shape(bad), "{bad} should be rejected");
        }
    }
}
