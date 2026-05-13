//! Session creation helpers.
//!
//! `create_session` is what every "the user is now logged in" code path runs:
//! generates a session id, signs a JWT, inserts a sessions row, and returns
//! both the JWT and the Set-Cookie value the handler should attach.

use axum_extra::extract::cookie::Cookie;
use chrono::{DateTime, TimeZone, Utc};
use sha2::{Digest, Sha256};
use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::cookie::session_cookie;
use crate::auth::jwt::{self, Claims, DEFAULT_TTL_DAYS};
use crate::error::{AppError, AppResult};

pub struct NewSession {
    pub cookie: Cookie<'static>,
}

pub async fn create_session(
    pool: &PgPool,
    user_id: Uuid,
    jwt_secret: &str,
) -> AppResult<NewSession> {
    let session_id = Uuid::new_v4();
    let claims = Claims::new(user_id, session_id, DEFAULT_TTL_DAYS);
    let token = jwt::sign(&claims, jwt_secret)?;
    let token_hash = sha256_hex(&token);
    let expires_at = Utc.timestamp_opt(claims.exp, 0).single().ok_or_else(|| {
        AppError::Internal(format!("session expiry out of range: {}", claims.exp))
    })?;

    sqlx::query(
        "INSERT INTO sessions (id, user_id, token_hash, expires_at) VALUES ($1, $2, $3, $4)",
    )
    .bind(session_id)
    .bind(user_id)
    .bind(&token_hash)
    .bind::<DateTime<Utc>>(expires_at)
    .execute(pool)
    .await?;

    Ok(NewSession {
        cookie: session_cookie(token),
    })
}

fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}
