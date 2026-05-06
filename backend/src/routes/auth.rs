//! Auth endpoints. See `docs/milestones/01_auth_and_accounts.md` §4.

use std::collections::HashMap;
use std::time::Duration as StdDuration;

use axum::{extract::State, http::HeaderMap, routing::post, Json, Router};
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::{
    code::six_digit_code,
    cookie::clear_session_cookie,
    extractor::AuthUser,
    password::{hash_password, verify_password},
    rate_limit::RateLimiter,
    session::create_session,
};
use crate::email::{email_change_verification, password_reset, verification, ResendClient};
use crate::error::{AppError, AppResult};
use crate::guest_state::{GuestState, MergeSummary};
use crate::onboarding;
use crate::state::AppState;

const VERIFICATION_TTL_MINUTES: i64 = 10;
const RESET_TTL_MINUTES: i64 = 60;

// Rate-limit windows. Documented in `docs/milestones/01_auth_and_accounts.md` §4.
const REGISTER_PER_IP: (usize, StdDuration) = (10, StdDuration::from_secs(60 * 60));
const LOGIN_PER_IP: (usize, StdDuration) = (20, StdDuration::from_secs(60));
const RESEND_PER_KEY: (usize, StdDuration) = (1, StdDuration::from_secs(60));
const FORGOT_PER_EMAIL: (usize, StdDuration) = (3, StdDuration::from_secs(60 * 60));

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/auth/register", post(register))
        .route("/auth/verify-email", post(verify_email))
        .route("/auth/resend-verification", post(resend_verification))
        .route("/auth/login", post(login))
        .route("/auth/logout", post(logout))
        .route("/auth/sign-out-all", post(sign_out_all))
        .route("/auth/forgot-password", post(forgot_password))
        .route("/auth/reset-password", post(reset_password))
        .route("/auth/change-password", post(change_password))
        .route("/auth/onboarding-complete", post(onboarding_complete))
        .route("/auth/merge-guest-state", post(merge_guest_state))
        .route(
            "/auth/me",
            axum::routing::get(me).patch(update_me),
        )
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
    /// Optional guest-mode state to import. When present, validated and
    /// imported in the same transaction as user creation — see
    /// `docs/milestones/06_guest_mode.md` §4.
    #[serde(default)]
    guest_state: Option<GuestState>,
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
    headers: HeaderMap,
    Json(req): Json<RegisterRequest>,
) -> AppResult<Json<RegisterResponse>> {
    let ip = client_ip(&headers);
    enforce_limit(
        &state.rate_limit,
        &format!("register:ip:{ip}"),
        REGISTER_PER_IP,
    )?;

    let display_name = req.display_name.trim().to_string();
    let email = req.email.trim().to_lowercase();

    validate_register(&display_name, &email, &req.password)?;
    state.recaptcha.verify(&req.recaptcha_token).await?;

    // Validate the guest blob (if any) up front so a malformed payload
    // never creates a user row in the first place.
    if let Some(ref gs) = req.guest_state {
        gs.validate()?;
    }

    let password_hash = hash_password(&req.password, state.argon2)?;
    let code = six_digit_code();
    let expires = Utc::now() + Duration::minutes(VERIFICATION_TTL_MINUTES);

    // Wrap user-creation + (optional) guest-state import in one transaction
    // so a mid-import failure rolls back the half-created user.
    let mut tx = state.pool.begin().await?;

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
    .fetch_one(&mut *tx)
    .await;

    let (id, email, display_name, email_verified) = match row {
        Ok(r) => r,
        Err(e) if is_unique_violation(&e) => return Err(AppError::EmailInUse),
        Err(e) => return Err(e.into()),
    };

    if let Some(ref gs) = req.guest_state {
        gs.import(&mut tx, id).await?;
    }

    tx.commit().await?;

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
    if let Some(user) = auth_user {
        enforce_limit(
            &state.rate_limit,
            &format!("resend:user:{}", user.user_id),
            RESEND_PER_KEY,
        )?;
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
        enforce_limit(
            &state.rate_limit,
            &format!("resend:email:{email}"),
            RESEND_PER_KEY,
        )?;
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
    headers: HeaderMap,
    jar: CookieJar,
    Json(req): Json<LoginRequest>,
) -> AppResult<(CookieJar, Json<LoginResponse>)> {
    let ip = client_ip(&headers);
    enforce_limit(&state.rate_limit, &format!("login:ip:{ip}"), LOGIN_PER_IP)?;

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
    has_seen_onboarding: bool,
}

async fn me(State(state): State<AppState>, user: AuthUser) -> AppResult<Json<MeResponse>> {
    let row: Option<(Uuid, String, Option<String>, String, bool, bool)> = sqlx::query_as(
        "SELECT id, email, pending_email, display_name, email_verified, has_seen_onboarding \
         FROM users WHERE id = $1",
    )
    .bind(user.user_id)
    .fetch_optional(&state.pool)
    .await?;

    let (id, email, pending_email, display_name, email_verified, has_seen_onboarding) =
        row.ok_or(AppError::Unauthorized)?;

    Ok(Json(MeResponse {
        id,
        email,
        pending_email,
        display_name,
        email_verified,
        has_seen_onboarding,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/sign-out-all
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct SignOutAllRequest {
    current_password: String,
}

async fn sign_out_all(
    State(state): State<AppState>,
    jar: CookieJar,
    user: AuthUser,
    Json(req): Json<SignOutAllRequest>,
) -> AppResult<(CookieJar, Json<EmptyResponse>)> {
    let row: Option<(String,)> =
        sqlx::query_as("SELECT password_hash FROM users WHERE id = $1")
            .bind(user.user_id)
            .fetch_optional(&state.pool)
            .await?;

    let (password_hash,) = row.ok_or(AppError::Unauthorized)?;
    if !verify_password(&req.current_password, &password_hash)? {
        return Err(AppError::InvalidPassword);
    }

    sqlx::query("UPDATE sessions SET revoked = TRUE WHERE user_id = $1")
        .bind(user.user_id)
        .execute(&state.pool)
        .await?;

    tracing::info!(user_id = %user.user_id, "user signed out everywhere");

    Ok((jar.add(clear_session_cookie()), Json(EmptyResponse {})))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/forgot-password
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ForgotPasswordRequest {
    email: String,
}

async fn forgot_password(
    State(state): State<AppState>,
    Json(req): Json<ForgotPasswordRequest>,
) -> AppResult<Json<EmptyResponse>> {
    let email = req.email.trim().to_lowercase();
    if email.is_empty() {
        let mut fields = HashMap::new();
        fields.insert("email".into(), "Required.".into());
        return Err(AppError::Validation(fields));
    }

    enforce_limit(
        &state.rate_limit,
        &format!("forgot:email:{email}"),
        FORGOT_PER_EMAIL,
    )?;

    let row: Option<(Uuid,)> = sqlx::query_as("SELECT id FROM users WHERE email = $1")
        .bind(&email)
        .fetch_optional(&state.pool)
        .await?;

    if let Some((id,)) = row {
        let code = six_digit_code();
        let expires = Utc::now() + Duration::minutes(RESET_TTL_MINUTES);
        sqlx::query(
            r#"
            UPDATE users
            SET reset_code = $2, reset_code_expires = $3
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(&code)
        .bind(expires)
        .execute(&state.pool)
        .await?;

        let msg = password_reset(&code);
        state
            .email
            .send(&email, &msg.subject, &msg.text, &msg.html)
            .await?;
    }
    // Always return 200 — don't reveal whether the email is registered.
    Ok(Json(EmptyResponse {}))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/reset-password
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ResetPasswordRequest {
    email: String,
    code: String,
    new_password: String,
}

async fn reset_password(
    State(state): State<AppState>,
    Json(req): Json<ResetPasswordRequest>,
) -> AppResult<Json<EmptyResponse>> {
    let email = req.email.trim().to_lowercase();
    let code = req.code.trim().to_string();

    let mut fields: HashMap<String, String> = HashMap::new();
    if email.is_empty() {
        fields.insert("email".into(), "Required.".into());
    }
    if code.len() != 6 || !code.chars().all(|c| c.is_ascii_digit()) {
        fields.insert("code".into(), "Enter the 6-digit code.".into());
    }
    if req.new_password.chars().count() < 8 {
        fields.insert("new_password".into(), "Must be at least 8 characters.".into());
    }
    if !fields.is_empty() {
        return Err(AppError::Validation(fields));
    }

    let row: Option<(Uuid, Option<String>, Option<DateTime<Utc>>)> = sqlx::query_as(
        "SELECT id, reset_code, reset_code_expires FROM users WHERE email = $1",
    )
    .bind(&email)
    .fetch_optional(&state.pool)
    .await?;

    let (id, stored_code, expires) = row.ok_or(AppError::InvalidCode)?;
    let stored = stored_code.ok_or(AppError::InvalidCode)?;
    let expires = expires.ok_or(AppError::InvalidCode)?;
    if stored != code {
        return Err(AppError::InvalidCode);
    }
    if expires <= Utc::now() {
        return Err(AppError::CodeExpired);
    }

    let new_hash = hash_password(&req.new_password, state.argon2)?;

    // Single transaction: store new password, clear reset code, revoke all sessions.
    // Revoking on reset is the right security default — the previous password
    // may be compromised, so any device still signed in needs to be kicked.
    let mut tx = state.pool.begin().await?;
    sqlx::query(
        r#"
        UPDATE users
        SET password_hash = $2, reset_code = NULL, reset_code_expires = NULL
        WHERE id = $1
        "#,
    )
    .bind(id)
    .bind(&new_hash)
    .execute(&mut *tx)
    .await?;
    sqlx::query("UPDATE sessions SET revoked = TRUE WHERE user_id = $1")
        .bind(id)
        .execute(&mut *tx)
        .await?;
    tx.commit().await?;

    tracing::info!(user_id = %id, "password reset, all sessions revoked");

    Ok(Json(EmptyResponse {}))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/change-password
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct ChangePasswordRequest {
    current_password: String,
    new_password: String,
}

async fn change_password(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<ChangePasswordRequest>,
) -> AppResult<Json<EmptyResponse>> {
    if req.new_password.chars().count() < 8 {
        let mut fields = HashMap::new();
        fields.insert("new_password".into(), "Must be at least 8 characters.".into());
        return Err(AppError::Validation(fields));
    }
    if req.new_password == req.current_password {
        let mut fields = HashMap::new();
        fields.insert(
            "new_password".into(),
            "Must differ from your current password.".into(),
        );
        return Err(AppError::Validation(fields));
    }

    let row: Option<(String,)> =
        sqlx::query_as("SELECT password_hash FROM users WHERE id = $1")
            .bind(user.user_id)
            .fetch_optional(&state.pool)
            .await?;
    let (current_hash,) = row.ok_or(AppError::Unauthorized)?;

    if !verify_password(&req.current_password, &current_hash)? {
        return Err(AppError::InvalidPassword);
    }

    let new_hash = hash_password(&req.new_password, state.argon2)?;
    sqlx::query("UPDATE users SET password_hash = $2 WHERE id = $1")
        .bind(user.user_id)
        .bind(&new_hash)
        .execute(&state.pool)
        .await?;

    tracing::info!(user_id = %user.user_id, "password changed");

    Ok(Json(EmptyResponse {}))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/onboarding-complete
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Serialize)]
pub struct OkResponse {
    ok: bool,
}

async fn onboarding_complete(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<OkResponse>> {
    onboarding::mark_seen(&state.pool, user.user_id).await?;
    Ok(Json(OkResponse { ok: true }))
}

// ─────────────────────────────────────────────────────────────────────────────
// POST /auth/merge-guest-state
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct MergeGuestStateRequest {
    guest_state: GuestState,
}

#[derive(Debug, Serialize)]
pub struct MergeGuestStateResponse {
    ok: bool,
    merged: MergeSummary,
}

async fn merge_guest_state(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<MergeGuestStateRequest>,
) -> AppResult<Json<MergeGuestStateResponse>> {
    req.guest_state.validate()?;

    let mut tx = state.pool.begin().await?;
    let summary = req.guest_state.merge(&mut tx, user.user_id).await?;
    tx.commit().await?;

    Ok(Json(MergeGuestStateResponse {
        ok: true,
        merged: summary,
    }))
}

// ─────────────────────────────────────────────────────────────────────────────
// PATCH /auth/me
// ─────────────────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
pub struct UpdateMeRequest {
    display_name: Option<String>,
    email: Option<String>,
}

async fn update_me(
    State(state): State<AppState>,
    user: AuthUser,
    Json(req): Json<UpdateMeRequest>,
) -> AppResult<Json<MeResponse>> {
    let mut fields: HashMap<String, String> = HashMap::new();

    if let Some(ref name) = req.display_name {
        let trimmed = name.trim();
        if trimmed.is_empty() {
            fields.insert("display_name".into(), "Required.".into());
        } else if trimmed.chars().count() > 80 {
            fields.insert(
                "display_name".into(),
                "Must be 80 characters or fewer.".into(),
            );
        }
    }
    if let Some(ref email) = req.email {
        let trimmed = email.trim();
        if !is_email_shape(trimmed) {
            fields.insert("email".into(), "Enter a valid email address.".into());
        }
    }
    if !fields.is_empty() {
        return Err(AppError::Validation(fields));
    }

    if let Some(name) = req.display_name.as_ref() {
        sqlx::query("UPDATE users SET display_name = $2 WHERE id = $1")
            .bind(user.user_id)
            .bind(name.trim())
            .execute(&state.pool)
            .await?;
    }

    if let Some(email_raw) = req.email.as_ref() {
        let new_email = email_raw.trim().to_lowercase();

        // Look up current email so we can compare and clear pending if reverting.
        let current: Option<(String,)> =
            sqlx::query_as("SELECT email FROM users WHERE id = $1")
                .bind(user.user_id)
                .fetch_optional(&state.pool)
                .await?;
        let (current_email,) = current.ok_or(AppError::Unauthorized)?;

        if new_email == current_email {
            // User reverted; clear any pending change.
            sqlx::query("UPDATE users SET pending_email = NULL WHERE id = $1")
                .bind(user.user_id)
                .execute(&state.pool)
                .await?;
        } else {
            // Reject if some other user already has this email.
            let collision: Option<(Uuid,)> = sqlx::query_as(
                "SELECT id FROM users WHERE email = $1 AND id <> $2",
            )
            .bind(&new_email)
            .bind(user.user_id)
            .fetch_optional(&state.pool)
            .await?;
            if collision.is_some() {
                return Err(AppError::EmailInUse);
            }

            let code = six_digit_code();
            let expires = Utc::now() + Duration::minutes(VERIFICATION_TTL_MINUTES);
            sqlx::query(
                r#"
                UPDATE users
                SET pending_email = $2,
                    verification_code = $3,
                    verification_code_expires = $4
                WHERE id = $1
                "#,
            )
            .bind(user.user_id)
            .bind(&new_email)
            .bind(&code)
            .bind(expires)
            .execute(&state.pool)
            .await?;

            let msg = email_change_verification(&code, &new_email);
            state
                .email
                .send(&new_email, &msg.subject, &msg.text, &msg.html)
                .await?;
        }
    }

    // Re-read and return the canonical record.
    let row: Option<(Uuid, String, Option<String>, String, bool, bool)> = sqlx::query_as(
        "SELECT id, email, pending_email, display_name, email_verified, has_seen_onboarding \
         FROM users WHERE id = $1",
    )
    .bind(user.user_id)
    .fetch_optional(&state.pool)
    .await?;
    let (id, email, pending_email, display_name, email_verified, has_seen_onboarding) =
        row.ok_or(AppError::Unauthorized)?;

    tracing::info!(user_id = %id, "profile updated");

    Ok(Json(MeResponse {
        id,
        email,
        pending_email,
        display_name,
        email_verified,
        has_seen_onboarding,
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

/// Best-effort client IP. Behind Render (and most reverse proxies) the
/// originating address shows up in `X-Forwarded-For`; the leftmost entry is
/// the real client. Falls back to `"unknown"` so the limiter still has a key
/// (worst case: all unknown-IP requests share a bucket).
fn client_ip(headers: &HeaderMap) -> String {
    if let Some(value) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        if let Some(first) = value.split(',').next() {
            let trimmed = first.trim();
            if !trimmed.is_empty() {
                return trimmed.to_string();
            }
        }
    }
    if let Some(value) = headers.get("x-real-ip").and_then(|v| v.to_str().ok()) {
        let trimmed = value.trim();
        if !trimmed.is_empty() {
            return trimmed.to_string();
        }
    }
    "unknown".to_string()
}

fn enforce_limit(
    limiter: &RateLimiter,
    key: &str,
    (limit, window): (usize, StdDuration),
) -> AppResult<()> {
    limiter
        .check(key, limit, window)
        .map_err(|retry_after| AppError::RateLimited { retry_after })
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
