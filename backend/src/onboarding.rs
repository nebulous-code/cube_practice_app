//! Onboarding-flag library functions. The route handler in
//! `routes::auth::onboarding_complete` delegates here so integration tests
//! can exercise the SQL behavior directly. See
//! `docs/milestones/05_polish_and_static_pages.md` §4.

use sqlx::PgPool;
use uuid::Uuid;

use crate::error::AppResult;

/// Mark the given user as having completed the post-verification onboarding
/// stub. Idempotent — calling twice has the same effect as calling once.
pub async fn mark_seen(pool: &PgPool, user_id: Uuid) -> AppResult<()> {
    sqlx::query("UPDATE users SET has_seen_onboarding = TRUE WHERE id = $1")
        .bind(user_id)
        .execute(pool)
        .await?;
    Ok(())
}
