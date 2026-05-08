//! Integration tests for `account_delete::delete_account`.
//! See docs/milestones/07_delete_account.md §7.
//!
//! Tests target `cube_backend::account_delete::delete_account`, the library
//! function the route handler delegates to. The HTTP auth gate + cookie wipe
//! are shared with every other authed endpoint and not retested here.

mod common;

use chrono::{DateTime, Utc};
use common::TestDb;
use cube_backend::account_delete;
use cube_backend::auth::password::{hash_password, Argon2Config};
use cube_backend::error::AppError;
use sqlx::PgPool;
use uuid::Uuid;

fn fast_config() -> Argon2Config {
    Argon2Config {
        m_kib: 8 * 1024,
        t: 1,
        p: 1,
    }
}

async fn seed_user(pool: &PgPool, email: &str, password: &str) -> Uuid {
    let hash = hash_password(password, fast_config()).expect("hash");
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, display_name, password_hash, email_verified) \
         VALUES ($1, 'Test', $2, TRUE) RETURNING id",
    )
    .bind(email)
    .bind(&hash)
    .fetch_one(pool)
    .await
    .expect("insert user");
    row.0
}

async fn user_exists(pool: &PgPool, user_id: Uuid) -> bool {
    let row: (i64,) = sqlx::query_as("SELECT COUNT(*) FROM users WHERE id = $1")
        .bind(user_id)
        .fetch_one(pool)
        .await
        .expect("count users");
    row.0 == 1
}

async fn audit_emails(pool: &PgPool) -> Vec<String> {
    let rows: Vec<(String,)> = sqlx::query_as(
        "SELECT email FROM account_deletions ORDER BY id ASC",
    )
    .fetch_all(pool)
    .await
    .expect("query audit");
    rows.into_iter().map(|(e,)| e).collect()
}

#[tokio::test]
async fn happy_path_deletes_user_and_writes_audit_row() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com", "correct-horse").await;

    account_delete::delete_account(&db.pool, user, "correct-horse")
        .await
        .expect("delete");

    assert!(!user_exists(&db.pool, user).await);
    assert_eq!(audit_emails(&db.pool).await, vec!["alice@example.com"]);
}

#[tokio::test]
async fn audit_row_carries_a_timestamp_close_to_now() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com", "correct-horse").await;
    let before = Utc::now();

    account_delete::delete_account(&db.pool, user, "correct-horse")
        .await
        .expect("delete");

    let row: (DateTime<Utc>,) =
        sqlx::query_as("SELECT deleted_at FROM account_deletions LIMIT 1")
            .fetch_one(&db.pool)
            .await
            .expect("query deleted_at");
    let after = Utc::now();
    assert!(row.0 >= before && row.0 <= after);
}

#[tokio::test]
async fn wrong_password_rejects_and_leaves_state_intact() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com", "correct-horse").await;

    let err = account_delete::delete_account(&db.pool, user, "wrong-password")
        .await
        .expect_err("should reject");

    assert!(matches!(err, AppError::InvalidPassword));
    assert!(user_exists(&db.pool, user).await);
    assert!(audit_emails(&db.pool).await.is_empty());
}

#[tokio::test]
async fn cross_user_isolation() {
    let db = TestDb::new().await;
    let alice = seed_user(&db.pool, "alice@example.com", "alice-pw").await;
    let bob = seed_user(&db.pool, "bob@example.com", "bob-pw").await;

    account_delete::delete_account(&db.pool, alice, "alice-pw")
        .await
        .expect("delete alice");

    assert!(!user_exists(&db.pool, alice).await);
    assert!(user_exists(&db.pool, bob).await);
    assert_eq!(audit_emails(&db.pool).await, vec!["alice@example.com"]);
}

#[tokio::test]
async fn cascade_removes_sessions_and_progress() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com", "pw").await;

    // Plant a session row + a progress row + a settings row, then assert
    // the cascade clears each one when the user is deleted.
    sqlx::query(
        "INSERT INTO sessions (user_id, token_hash, expires_at) \
         VALUES ($1, $2, now() + interval '1 day')",
    )
    .bind(user)
    .bind(Uuid::new_v4().to_string())
    .execute(&db.pool)
    .await
    .expect("seed session");

    let case_id: (Uuid,) = sqlx::query_as("SELECT id FROM cases LIMIT 1")
        .fetch_one(&db.pool)
        .await
        .expect("fetch a case");

    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, nickname) \
         VALUES ($1, $2, 'override')",
    )
    .bind(user)
    .bind(case_id.0)
    .execute(&db.pool)
    .await
    .expect("seed settings");

    sqlx::query(
        "INSERT INTO user_case_progress (user_id, case_id, repetitions, ease_factor, interval_days, due_date) \
         VALUES ($1, $2, 1, 2.5, 1, CURRENT_DATE)",
    )
    .bind(user)
    .bind(case_id.0)
    .execute(&db.pool)
    .await
    .expect("seed progress");

    account_delete::delete_account(&db.pool, user, "pw")
        .await
        .expect("delete");

    let session_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM sessions WHERE user_id = $1")
            .bind(user)
            .fetch_one(&db.pool)
            .await
            .expect("count sessions");
    let settings_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM user_case_settings WHERE user_id = $1")
            .bind(user)
            .fetch_one(&db.pool)
            .await
            .expect("count settings");
    let progress_count: (i64,) =
        sqlx::query_as("SELECT COUNT(*) FROM user_case_progress WHERE user_id = $1")
            .bind(user)
            .fetch_one(&db.pool)
            .await
            .expect("count progress");

    assert_eq!(session_count.0, 0);
    assert_eq!(settings_count.0, 0);
    assert_eq!(progress_count.0, 0);
}

#[tokio::test]
async fn unknown_user_returns_unauthorized() {
    let db = TestDb::new().await;

    let err = account_delete::delete_account(&db.pool, Uuid::new_v4(), "anything")
        .await
        .expect_err("should reject");
    assert!(matches!(err, AppError::Unauthorized));
}

#[tokio::test]
async fn re_register_after_delete_starts_with_fresh_streak() {
    let db = TestDb::new().await;
    let first = seed_user(&db.pool, "alice@example.com", "pw1").await;

    // Simulate a few days of practice on the first account.
    sqlx::query(
        "UPDATE users SET streak_count = 2, last_practice_date = CURRENT_DATE WHERE id = $1",
    )
    .bind(first)
    .execute(&db.pool)
    .await
    .expect("seed streak");

    account_delete::delete_account(&db.pool, first, "pw1")
        .await
        .expect("delete");

    // Re-register the same email — should be a fresh row with default streak.
    let second = seed_user(&db.pool, "alice@example.com", "pw2").await;
    let row: (i32, Option<chrono::NaiveDate>) = sqlx::query_as(
        "SELECT streak_count, last_practice_date FROM users WHERE id = $1",
    )
    .bind(second)
    .fetch_one(&db.pool)
    .await
    .expect("read fresh user");

    assert_eq!(row.0, 0, "fresh user should start with streak_count=0");
    assert!(row.1.is_none(), "fresh user should start with last_practice_date=NULL");
    assert_ne!(first, second, "re-register should produce a new user id");
}

#[tokio::test]
async fn re_register_then_delete_creates_separate_audit_rows() {
    let db = TestDb::new().await;
    let first = seed_user(&db.pool, "alice@example.com", "pw1").await;
    account_delete::delete_account(&db.pool, first, "pw1")
        .await
        .expect("first delete");

    let second = seed_user(&db.pool, "alice@example.com", "pw2").await;
    account_delete::delete_account(&db.pool, second, "pw2")
        .await
        .expect("second delete");

    assert_eq!(
        audit_emails(&db.pool).await,
        vec!["alice@example.com".to_string(), "alice@example.com".to_string()]
    );
}
