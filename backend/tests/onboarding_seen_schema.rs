//! Schema-level checks for the M5 has_seen_onboarding migration. Verifies
//! the column exists, is NOT NULL with DEFAULT FALSE, and that fresh user
//! rows backfill / insert as FALSE.

mod common;

use common::TestDb;
use sqlx::Row;
use uuid::Uuid;

#[tokio::test]
async fn has_seen_onboarding_is_not_null_with_false_default() {
    let db = TestDb::new().await;

    let row = sqlx::query(
        "SELECT is_nullable, column_default, data_type FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'users' \
           AND column_name = 'has_seen_onboarding'",
    )
    .fetch_one(&db.pool)
    .await
    .expect("query column metadata");

    let nullable: String = row.get("is_nullable");
    let default: Option<String> = row.get("column_default");
    let data_type: String = row.get("data_type");

    assert_eq!(nullable, "NO", "has_seen_onboarding should be NOT NULL");
    assert_eq!(data_type, "boolean");
    let default = default.expect("has_seen_onboarding should have a default");
    assert!(
        default.starts_with("false"),
        "expected DEFAULT FALSE, got: {default}"
    );
}

#[tokio::test]
async fn fresh_user_starts_with_has_seen_onboarding_false() {
    let db = TestDb::new().await;

    let row: (Uuid, bool) = sqlx::query_as(
        "INSERT INTO users (email, display_name, password_hash) \
         VALUES ($1, 'Test', 'x') RETURNING id, has_seen_onboarding",
    )
    .bind("alice@example.com")
    .fetch_one(&db.pool)
    .await
    .expect("insert user");

    assert!(!row.1, "fresh user should have has_seen_onboarding = false");
}
