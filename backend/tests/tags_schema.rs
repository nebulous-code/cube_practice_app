//! Schema-level checks for the M4 tags-array migration. Verifies the
//! tier2_tag → tags TEXT[] conversion: seed values backfilled as
//! one-element arrays, cases.tags is NOT NULL with default '{}',
//! user_case_settings.tags is nullable, the GIN index exists, and the
//! old tier2_tag column is gone from both tables.

mod common;

use common::TestDb;
use sqlx::Row;
use uuid::Uuid;

#[tokio::test]
async fn tier2_tag_column_is_gone_from_both_tables() {
    let db = TestDb::new().await;

    for table in ["cases", "user_case_settings"] {
        let row: (i64,) = sqlx::query_as(
            "SELECT count(*)::bigint FROM information_schema.columns \
             WHERE table_schema = 'public' AND table_name = $1 AND column_name = 'tier2_tag'",
        )
        .bind(table)
        .fetch_one(&db.pool)
        .await
        .expect("information_schema query");
        assert_eq!(row.0, 0, "expected tier2_tag to be dropped from {table}");
    }
}

#[tokio::test]
async fn cases_tags_is_not_null_with_empty_array_default() {
    let db = TestDb::new().await;

    let row = sqlx::query(
        "SELECT is_nullable, column_default FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'cases' AND column_name = 'tags'",
    )
    .fetch_one(&db.pool)
    .await
    .expect("query column metadata");

    let nullable: String = row.get("is_nullable");
    let default: Option<String> = row.get("column_default");
    assert_eq!(nullable, "NO", "cases.tags should be NOT NULL");
    let default = default.expect("cases.tags should have a default");
    assert!(
        default.starts_with("'{}'"),
        "cases.tags default should be '{{}}', got: {default}"
    );
}

#[tokio::test]
async fn user_case_settings_tags_is_nullable() {
    let db = TestDb::new().await;

    let row = sqlx::query(
        "SELECT is_nullable FROM information_schema.columns \
         WHERE table_schema = 'public' AND table_name = 'user_case_settings' \
           AND column_name = 'tags'",
    )
    .fetch_one(&db.pool)
    .await
    .expect("query column metadata");

    let nullable: String = row.get("is_nullable");
    assert_eq!(nullable, "YES", "user_case_settings.tags should be nullable");
}

#[tokio::test]
async fn seeded_case_with_tier2_tag_lands_as_one_element_array() {
    let db = TestDb::new().await;

    // Case 06 in the seed has tier2_tag = 'fish' (a Fish-shape OLL).
    // Pick any seeded case and assert its tags array is non-empty.
    let row = sqlx::query(
        "SELECT case_number, tags FROM cases \
         WHERE array_length(tags, 1) IS NOT NULL \
         ORDER BY case_number ASC LIMIT 1",
    )
    .fetch_one(&db.pool)
    .await
    .expect("expected at least one seeded case to have tags");

    let tags: Vec<String> = row.get("tags");
    assert!(!tags.is_empty(), "expected non-empty tags after backfill");
}

#[tokio::test]
async fn seeded_case_without_tier2_tag_lands_as_empty_array() {
    let db = TestDb::new().await;

    // Some seed rows have NULL tier2_tag (the OCLL / "solves" group, etc.).
    // After the migration those rows must have an empty array, not NULL.
    let row: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM cases WHERE tags IS NULL",
    )
    .fetch_one(&db.pool)
    .await
    .expect("count");
    assert_eq!(row.0, 0, "no case row should have NULL tags");
}

#[tokio::test]
async fn user_case_settings_tags_starts_null() {
    // A fresh user override row created without specifying tags should
    // have tags = NULL (no override → fall back to global).
    let db = TestDb::new().await;
    let user_id = seed_user(&db.pool, "alice@example.com").await;

    let case_id: (Uuid,) = sqlx::query_as(
        "SELECT id FROM cases ORDER BY case_number ASC LIMIT 1",
    )
    .fetch_one(&db.pool)
    .await
    .expect("select first seeded case");

    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, nickname) VALUES ($1, $2, 'Mine')",
    )
    .bind(user_id)
    .bind(case_id.0)
    .execute(&db.pool)
    .await
    .expect("insert override");

    let row = sqlx::query(
        "SELECT tags FROM user_case_settings WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user_id)
    .bind(case_id.0)
    .fetch_one(&db.pool)
    .await
    .expect("select override");

    let tags: Option<Vec<String>> = row.get("tags");
    assert!(tags.is_none(), "expected NULL tags on a fresh override, got {tags:?}");
}

#[tokio::test]
async fn user_case_settings_tags_round_trip() {
    let db = TestDb::new().await;
    let user_id = seed_user(&db.pool, "bob@example.com").await;

    let case_id: (Uuid,) = sqlx::query_as(
        "SELECT id FROM cases ORDER BY case_number ASC LIMIT 1",
    )
    .fetch_one(&db.pool)
    .await
    .expect("select first seeded case");

    let written = vec!["fish".to_string(), "needs work".to_string()];
    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, tags) VALUES ($1, $2, $3)",
    )
    .bind(user_id)
    .bind(case_id.0)
    .bind(&written)
    .execute(&db.pool)
    .await
    .expect("insert override");

    let row = sqlx::query(
        "SELECT tags FROM user_case_settings WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user_id)
    .bind(case_id.0)
    .fetch_one(&db.pool)
    .await
    .expect("select override");

    let tags: Option<Vec<String>> = row.get("tags");
    assert_eq!(tags, Some(written));
}

#[tokio::test]
async fn cases_tags_gin_index_exists() {
    let db = TestDb::new().await;

    let row: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM pg_indexes \
         WHERE schemaname = 'public' AND indexname = 'cases_tags_gin_idx'",
    )
    .fetch_one(&db.pool)
    .await
    .expect("pg_indexes query");
    assert_eq!(row.0, 1, "expected cases_tags_gin_idx to exist");
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

async fn seed_user(pool: &sqlx::PgPool, email: &str) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, display_name, password_hash) VALUES ($1, 'Test', 'x') RETURNING id",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .expect("insert user");

    row.0
}
