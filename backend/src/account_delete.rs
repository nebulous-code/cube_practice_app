//! Account-deletion library function. The route handler in
//! `routes::auth::delete_account` delegates here so integration tests can
//! exercise the SQL behavior directly. See
//! `docs/milestones/07_delete_account.md` §3 + §6.

use sqlx::PgPool;
use uuid::Uuid;

use crate::auth::password::verify_password;
use crate::error::{AppError, AppResult};

/// Verify the caller's current password, then in a single transaction:
/// 1. Insert an `account_deletions` audit row capturing email + timestamp.
/// 2. Delete the `users` row — `ON DELETE CASCADE` cleans up sessions,
///    user_case_settings, and user_case_progress.
///
/// The audit insert + DELETE share a transaction so we can never end up
/// with a deleted user lacking an audit row, or an audit row whose user
/// rollback prevented the actual delete.
pub async fn delete_account(
    pool: &PgPool,
    user_id: Uuid,
    current_password: &str,
) -> AppResult<()> {
    let row: Option<(String, String)> =
        sqlx::query_as("SELECT email, password_hash FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_optional(pool)
            .await?;

    let (email, password_hash) = row.ok_or(AppError::Unauthorized)?;

    if !verify_password(current_password, &password_hash)? {
        return Err(AppError::InvalidPassword);
    }

    let mut tx = pool.begin().await?;

    sqlx::query("INSERT INTO account_deletions (email) VALUES ($1)")
        .bind(&email)
        .execute(&mut *tx)
        .await?;

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&mut *tx)
        .await?;

    tx.commit().await?;

    Ok(())
}
