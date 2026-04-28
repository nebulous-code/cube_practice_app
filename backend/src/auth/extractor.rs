//! `AuthUser` extractor — pulls the JWT cookie off a request, validates it,
//! and confirms the matching `sessions` row is still active.
//!
//! Per `docs/milestones/01_auth_and_accounts.md` §4 ("Per-Request Auth").

use axum::{
    extract::FromRequestParts,
    http::request::Parts,
};
use axum_extra::extract::cookie::CookieJar;
use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::auth::jwt::{decode_token, Claims};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

/// Cookie name for the session JWT. Matches the spec.
pub const SESSION_COOKIE: &str = "cube_session";

/// Authenticated user injected into protected handlers.
#[derive(Debug, Clone)]
pub struct AuthUser {
    pub user_id: Uuid,
    pub session_id: Uuid,
}

impl AuthUser {
    /// Look the user up from a request, given app state. Pulled out as a free
    /// function so optional-auth handlers (verify-email, resend-verification)
    /// can reuse the logic without going through the FromRequestParts trait
    /// (which only supports the required-auth case).
    pub async fn from_parts(parts: &Parts, state: &AppState) -> AppResult<Self> {
        let jar = CookieJar::from_headers(&parts.headers);
        let token = jar
            .get(SESSION_COOKIE)
            .map(|c| c.value().to_string())
            .ok_or(AppError::Unauthorized)?;

        let claims: Claims = decode_token(&token, &state.config.jwt_secret)?;
        verify_session_active(&state.pool, claims.sid).await?;

        Ok(Self {
            user_id: claims.sub,
            session_id: claims.sid,
        })
    }

    /// Optional version: returns Ok(None) if there's no cookie or it fails validation.
    /// Used by endpoints that behave differently depending on auth state.
    pub async fn try_from_parts(parts: &Parts, state: &AppState) -> Option<Self> {
        Self::from_parts(parts, state).await.ok()
    }
}

#[axum::async_trait]
impl FromRequestParts<AppState> for AuthUser {
    type Rejection = AppError;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        Self::from_parts(parts, state).await
    }
}

/// Confirm the `sessions` row for `session_id` is not revoked and not expired.
/// Anything else collapses to `Unauthorized` so we don't leak failure mode.
///
/// Untyped `sqlx::query_as` (rather than the `query!` macro) so the crate
/// builds without a live database at compile time — important for Render's
/// build environment where Neon isn't reachable from the build container.
async fn verify_session_active(pool: &sqlx::PgPool, session_id: Uuid) -> AppResult<()> {
    let row: Option<(bool, DateTime<Utc>)> = sqlx::query_as(
        "SELECT revoked, expires_at FROM sessions WHERE id = $1",
    )
    .bind(session_id)
    .fetch_optional(pool)
    .await?;

    match row {
        Some((revoked, expires_at)) if !revoked && expires_at > Utc::now() => Ok(()),
        _ => Err(AppError::Unauthorized),
    }
}
