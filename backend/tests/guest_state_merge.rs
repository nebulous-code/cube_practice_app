//! M6 B4 — `POST /auth/merge-guest-state` merge logic. Drives
//! `GuestState::merge` directly (mirrors B3's pattern). See
//! docs/milestones/06_guest_mode.md §4.
//!
//! Merge rules covered here:
//!   - Settings: skip when server already has an override (no overwrite)
//!   - Progress: keep higher interval_days; ties → server; insert when no
//!     server row
//!   - Streak: max(server, guest); last_practice_date = later of the two
//!   - Onboarding: OR
//!   - Tag caps: enforced over the union of server + guest tags

mod common;

use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use common::TestDb;
use cube_backend::cases::MAX_DISTINCT_TAGS_PER_USER;
use cube_backend::error::AppError;
use cube_backend::guest_state::{
    GuestProgress, GuestSettings, GuestState, SCHEMA_VERSION,
};
use sqlx::PgPool;
use uuid::Uuid;

async fn seed_user(pool: &PgPool, email: &str) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, display_name, password_hash) \
         VALUES ($1, 'Test', 'x') RETURNING id",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .expect("insert user");
    row.0
}

async fn case_id_for(pool: &PgPool, n: i32) -> Uuid {
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM cases WHERE case_number = $1")
        .bind(n)
        .fetch_one(pool)
        .await
        .expect("case lookup");
    row.0
}

fn empty_state() -> GuestState {
    GuestState {
        version: SCHEMA_VERSION,
        display_name: None,
        created_at: None,
        streak_count: 0,
        last_practice_date: None,
        onboarding_completed: false,
        settings: HashMap::new(),
        progress: HashMap::new(),
    }
}

async fn merge(
    pool: &PgPool,
    user_id: Uuid,
    state: &GuestState,
) -> Result<cube_backend::guest_state::MergeSummary, AppError> {
    let mut tx = pool.begin().await.unwrap();
    let result = state.merge(&mut tx, user_id).await;
    if result.is_ok() {
        tx.commit().await.unwrap();
    }
    result
}

// ─── Settings: server wins on collision ─────────────────────────────────────

#[tokio::test]
async fn merge_skips_settings_when_server_override_exists() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let c = case_id_for(&db.pool, 12).await;

    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, nickname) VALUES ($1, $2, 'Server Pick')",
    )
    .bind(user)
    .bind(c)
    .execute(&db.pool)
    .await
    .unwrap();

    let mut state = empty_state();
    state.settings.insert(
        "12".into(),
        GuestSettings {
            nickname: Some("Guest Pick".into()),
            algorithm: None,
            result_case_number: None,
            result_rotation: None,
            tags: vec![],
        },
    );

    let summary = merge(&db.pool, user, &state).await.unwrap();
    assert_eq!(summary.cases, 0);

    let row: (Option<String>,) = sqlx::query_as(
        "SELECT nickname FROM user_case_settings WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user)
    .bind(c)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(row.0.as_deref(), Some("Server Pick"));
}

#[tokio::test]
async fn merge_inserts_settings_when_no_server_override() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let c = case_id_for(&db.pool, 12).await;

    let mut state = empty_state();
    state.settings.insert(
        "12".into(),
        GuestSettings {
            nickname: Some("Guest Pick".into()),
            algorithm: None,
            result_case_number: None,
            result_rotation: None,
            tags: vec!["fish".into()],
        },
    );

    let summary = merge(&db.pool, user, &state).await.unwrap();
    assert_eq!(summary.cases, 1);

    let row: (Option<String>, Option<Vec<String>>) = sqlx::query_as(
        "SELECT nickname, tags FROM user_case_settings WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user)
    .bind(c)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(row.0.as_deref(), Some("Guest Pick"));
    assert_eq!(row.1, Some(vec!["fish".into()]));
}

// ─── Progress: max interval wins; ties go to server ─────────────────────────

#[tokio::test]
async fn merge_progress_keeps_higher_interval_when_server_wins() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let c = case_id_for(&db.pool, 5).await;

    // Server: interval=10. Guest: interval=2 (lower). Server wins.
    sqlx::query(
        "INSERT INTO user_case_progress (user_id, case_id, ease_factor, interval_days, repetitions, due_date) \
         VALUES ($1, $2, 2.5, 10, 3, $3)",
    )
    .bind(user)
    .bind(c)
    .bind(NaiveDate::from_ymd_opt(2026, 7, 1).unwrap())
    .execute(&db.pool)
    .await
    .unwrap();

    let mut state = empty_state();
    state.progress.insert(
        "5".into(),
        GuestProgress {
            ease_factor: 2.0,
            interval_days: 2,
            repetitions: 1,
            due_date: NaiveDate::from_ymd_opt(2026, 5, 12).unwrap(),
            last_grade: Some(0),
            last_reviewed: Some(Utc::now()),
        },
    );

    merge(&db.pool, user, &state).await.unwrap();

    let row: (i32,) = sqlx::query_as(
        "SELECT interval_days FROM user_case_progress WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user)
    .bind(c)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(row.0, 10, "server's higher interval must win");
}

#[tokio::test]
async fn merge_progress_takes_guest_when_guest_interval_is_higher() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let c = case_id_for(&db.pool, 5).await;

    // Server: interval=2. Guest: interval=10 (higher). Guest wins.
    sqlx::query(
        "INSERT INTO user_case_progress (user_id, case_id, ease_factor, interval_days, repetitions, due_date) \
         VALUES ($1, $2, 2.5, 2, 1, $3)",
    )
    .bind(user)
    .bind(c)
    .bind(NaiveDate::from_ymd_opt(2026, 5, 12).unwrap())
    .execute(&db.pool)
    .await
    .unwrap();

    let mut state = empty_state();
    state.progress.insert(
        "5".into(),
        GuestProgress {
            ease_factor: 2.7,
            interval_days: 10,
            repetitions: 4,
            due_date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
            last_grade: Some(2),
            last_reviewed: Some(Utc::now()),
        },
    );

    merge(&db.pool, user, &state).await.unwrap();

    let row: (i32, i32, NaiveDate) = sqlx::query_as(
        "SELECT interval_days, repetitions, due_date FROM user_case_progress \
         WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user)
    .bind(c)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(row.0, 10);
    assert_eq!(row.1, 4);
    assert_eq!(row.2, NaiveDate::from_ymd_opt(2026, 7, 1).unwrap());
}

#[tokio::test]
async fn merge_progress_ties_keep_server() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let c = case_id_for(&db.pool, 5).await;

    sqlx::query(
        "INSERT INTO user_case_progress (user_id, case_id, ease_factor, interval_days, repetitions, due_date) \
         VALUES ($1, $2, 2.4, 6, 2, $3)",
    )
    .bind(user)
    .bind(c)
    .bind(NaiveDate::from_ymd_opt(2026, 5, 30).unwrap())
    .execute(&db.pool)
    .await
    .unwrap();

    let mut state = empty_state();
    state.progress.insert(
        "5".into(),
        GuestProgress {
            ease_factor: 3.0, // would-be different value
            interval_days: 6, // tie
            repetitions: 5,
            due_date: NaiveDate::from_ymd_opt(2030, 1, 1).unwrap(),
            last_grade: Some(3),
            last_reviewed: Some(Utc::now()),
        },
    );

    merge(&db.pool, user, &state).await.unwrap();

    let row: (f64, i32) = sqlx::query_as(
        "SELECT ease_factor, repetitions FROM user_case_progress \
         WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user)
    .bind(c)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert!((row.0 - 2.4).abs() < 1e-9, "server's ease must be preserved on tie");
    assert_eq!(row.1, 2, "server's repetitions must be preserved on tie");
}

#[tokio::test]
async fn merge_progress_inserts_when_server_has_no_row() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let c = case_id_for(&db.pool, 5).await;

    let mut state = empty_state();
    state.progress.insert(
        "5".into(),
        GuestProgress {
            ease_factor: 2.5,
            interval_days: 3,
            repetitions: 1,
            due_date: NaiveDate::from_ymd_opt(2026, 5, 20).unwrap(),
            last_grade: Some(2),
            last_reviewed: Some(Utc::now()),
        },
    );

    merge(&db.pool, user, &state).await.unwrap();

    let row: (i32,) = sqlx::query_as(
        "SELECT interval_days FROM user_case_progress \
         WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user)
    .bind(c)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(row.0, 3);
}

// ─── Streak ─────────────────────────────────────────────────────────────────

#[tokio::test]
async fn merge_streak_takes_max() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    sqlx::query("UPDATE users SET streak_count = 5, last_practice_date = $2 WHERE id = $1")
        .bind(user)
        .bind(NaiveDate::from_ymd_opt(2026, 5, 1).unwrap())
        .execute(&db.pool)
        .await
        .unwrap();

    let mut state = empty_state();
    state.streak_count = 12;
    state.last_practice_date = NaiveDate::from_ymd_opt(2026, 4, 28); // earlier date

    merge(&db.pool, user, &state).await.unwrap();

    let row: (i32, NaiveDate) =
        sqlx::query_as("SELECT streak_count, last_practice_date FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(row.0, 12, "max streak wins");
    assert_eq!(
        row.1,
        NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
        "later date wins (server's was later)"
    );
}

#[tokio::test]
async fn merge_streak_with_null_server_date_takes_guest() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    // last_practice_date defaults to NULL for fresh user.

    let mut state = empty_state();
    state.streak_count = 3;
    state.last_practice_date = NaiveDate::from_ymd_opt(2026, 5, 1);

    merge(&db.pool, user, &state).await.unwrap();

    let row: (i32, Option<NaiveDate>) =
        sqlx::query_as("SELECT streak_count, last_practice_date FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(row.0, 3);
    assert_eq!(row.1, NaiveDate::from_ymd_opt(2026, 5, 1));
}

// ─── Onboarding flag ────────────────────────────────────────────────────────

#[tokio::test]
async fn merge_onboarding_ors_with_server_flag() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let mut state = empty_state();
    state.onboarding_completed = true;

    merge(&db.pool, user, &state).await.unwrap();

    let row: (bool,) =
        sqlx::query_as("SELECT has_seen_onboarding FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert!(row.0);
}

// ─── Tag caps ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn merge_rejects_when_union_exceeds_tag_cap() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let c1 = case_id_for(&db.pool, 1).await;

    // Server has 99 distinct tags on case 1.
    let server_tags: Vec<String> =
        (0..MAX_DISTINCT_TAGS_PER_USER - 1).map(|i| format!("s{i}")).collect();
    sqlx::query("INSERT INTO user_case_settings (user_id, case_id, tags) VALUES ($1, $2, $3)")
        .bind(user)
        .bind(c1)
        .bind(&server_tags)
        .execute(&db.pool)
        .await
        .unwrap();

    // Guest brings 5 new distinct tags on case 2 — union = 104, over cap.
    let mut state = empty_state();
    state.settings.insert(
        "2".into(),
        GuestSettings {
            nickname: None,
            algorithm: None,
            result_case_number: None,
            result_rotation: None,
            tags: (0..5).map(|i| format!("g{i}")).collect(),
        },
    );

    let err = merge(&db.pool, user, &state).await.expect_err("over cap rejects");
    match err {
        AppError::Validation(_) => {}
        other => panic!("expected Validation, got {other:?}"),
    }
}

// ─── No-op merge ────────────────────────────────────────────────────────────

#[tokio::test]
async fn merge_empty_blob_is_a_noop() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    sqlx::query("UPDATE users SET streak_count = 7 WHERE id = $1")
        .bind(user)
        .execute(&db.pool)
        .await
        .unwrap();

    let summary = merge(&db.pool, user, &empty_state()).await.unwrap();
    assert_eq!(summary.cases, 0);
    assert_eq!(summary.tags, 0);

    let row: (i32,) = sqlx::query_as("SELECT streak_count FROM users WHERE id = $1")
        .bind(user)
        .fetch_one(&db.pool)
        .await
        .unwrap();
    assert_eq!(row.0, 7, "empty blob must not regress streak");
}

// ─── Defense-in-depth — merge() called without validate() first ─────────────
//
// Mirrors the equivalent guard tests in guest_state_import.rs. validate() is
// the contract gate, but `merge` is `pub`; these pin down the structured
// errors that surface when callers feed it malformed/inconsistent input.

fn empty_settings() -> GuestSettings {
    GuestSettings {
        nickname: None,
        algorithm: None,
        result_case_number: None,
        result_rotation: None,
        tags: vec![],
    }
}

#[tokio::test]
async fn merge_rejects_non_numeric_settings_key() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let mut state = empty_state();
    state.settings.insert("not-a-number".into(), empty_settings());

    let err = merge(&db.pool, user, &state).await.expect_err("rejects");
    match err {
        AppError::Validation(fields) => {
            assert!(fields.contains_key("guest_state.settings"), "{fields:?}");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[tokio::test]
async fn merge_rejects_non_numeric_progress_key() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let mut state = empty_state();
    state.progress.insert(
        "not-a-number".into(),
        GuestProgress {
            ease_factor: 2.5,
            interval_days: 1,
            repetitions: 0,
            due_date: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            last_grade: None,
            last_reviewed: None,
        },
    );

    let err = merge(&db.pool, user, &state).await.expect_err("rejects");
    match err {
        AppError::Validation(fields) => {
            assert!(fields.contains_key("guest_state.progress"), "{fields:?}");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

async fn delete_case(pool: &PgPool, case_number: i32) {
    sqlx::query("UPDATE cases SET result_case_id = NULL")
        .execute(pool)
        .await
        .expect("null result_case_ids");
    sqlx::query("DELETE FROM cases WHERE case_number = $1")
        .bind(case_number)
        .execute(pool)
        .await
        .expect("delete case");
}

#[tokio::test]
async fn merge_rejects_settings_pointing_at_missing_case() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    delete_case(&db.pool, 5).await;

    let mut state = empty_state();
    state.settings.insert(
        "5".into(),
        GuestSettings {
            nickname: Some("Mine".into()),
            ..empty_settings()
        },
    );

    let err = merge(&db.pool, user, &state).await.expect_err("rejects");
    match err {
        AppError::Validation(fields) => {
            assert!(fields.contains_key("guest_state.settings"), "{fields:?}");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[tokio::test]
async fn merge_rejects_settings_with_missing_result_case() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    delete_case(&db.pool, 30).await;

    let mut state = empty_state();
    state.settings.insert(
        "1".into(),
        GuestSettings {
            result_case_number: Some(30),
            ..empty_settings()
        },
    );

    let err = merge(&db.pool, user, &state).await.expect_err("rejects");
    match err {
        AppError::Validation(fields) => {
            assert!(
                fields.contains_key("guest_state.settings.result_case_number"),
                "{fields:?}",
            );
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[tokio::test]
async fn merge_skips_settings_rows_with_no_real_overrides() {
    // Mirror of import's empty-override skip path. A guest blob may carry
    // entries for cases the user only browsed without overriding; merge
    // should drop those and not insert empty rows.
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let mut state = empty_state();
    state.settings.insert("1".into(), empty_settings());
    state
        .settings
        .insert("2".into(), empty_settings()); // also empty

    let summary = merge(&db.pool, user, &state).await.unwrap();
    assert_eq!(summary.cases, 0, "no real overrides should land");

    let count: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM user_case_settings WHERE user_id = $1",
    )
    .bind(user)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(count.0, 0);
}

#[tokio::test]
async fn merge_rejects_when_union_exceeds_link_cap() {
    // Trips the total_links cap (per-tag-link ceiling) without exceeding
    // the distinct cap. The existing `merge_rejects_when_union_exceeds_tag_cap`
    // test trips the distinct cap; this covers the link-cap branch.
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    // 50 cases × 21 tags each = 1050 links — over the 1000 link cap.
    // Tags repeat across cases so distinct stays well under the 100 cap.
    let mut state = empty_state();
    for n in 1..=50i32 {
        let tags: Vec<String> = (0..21).map(|i| format!("p{i}")).collect();
        state.settings.insert(
            n.to_string(),
            GuestSettings {
                tags,
                ..empty_settings()
            },
        );
    }

    let err = merge(&db.pool, user, &state).await.expect_err("rejects");
    match err {
        AppError::Validation(fields) => {
            assert!(
                fields.contains_key("guest_state.settings.tags"),
                "{fields:?}",
            );
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[tokio::test]
async fn merge_rejects_progress_pointing_at_missing_case() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    delete_case(&db.pool, 5).await;

    let mut state = empty_state();
    state.progress.insert(
        "5".into(),
        GuestProgress {
            ease_factor: 2.5,
            interval_days: 3,
            repetitions: 2,
            due_date: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
            last_grade: Some(2),
            last_reviewed: None,
        },
    );

    let err = merge(&db.pool, user, &state).await.expect_err("rejects");
    match err {
        AppError::Validation(fields) => {
            assert!(fields.contains_key("guest_state.progress"), "{fields:?}");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}
