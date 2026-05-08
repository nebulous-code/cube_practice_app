//! M6 B3 — guest_state validation + transactional import. The HTTP layer
//! (POST /auth/register with guest_state) wraps GuestState::import in a
//! transaction; these tests exercise the import path directly. See
//! docs/milestones/06_guest_mode.md §4 + §7.

mod common;

use std::collections::HashMap;

use chrono::{NaiveDate, Utc};
use common::TestDb;
use cube_backend::cases::{MAX_DISTINCT_TAGS_PER_USER, MAX_TAG_LINKS_PER_USER};
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

async fn import(pool: &PgPool, user_id: Uuid, state: &GuestState) -> Result<(), AppError> {
    let mut tx = pool.begin().await.expect("begin tx");
    let result = state.import(&mut tx, user_id).await;
    if result.is_ok() {
        tx.commit().await.expect("commit");
    } else {
        // Don't bother committing on error — sqlx rolls back on drop.
    }
    result
}

// ─── Happy path ──────────────────────────────────────────────────────────────

#[tokio::test]
async fn import_empty_state_succeeds_no_rows_written() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    import(&db.pool, user, &empty_state()).await.unwrap();

    let settings_count: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM user_case_settings WHERE user_id = $1",
    )
    .bind(user)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(settings_count.0, 0);

    let progress_count: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM user_case_progress WHERE user_id = $1",
    )
    .bind(user)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(progress_count.0, 0);
}

#[tokio::test]
async fn import_settings_lands_with_overrides_translated() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let mut state = empty_state();
    state.settings.insert(
        "12".into(),
        GuestSettings {
            nickname: Some("Slash".into()),
            algorithm: Some("F R U R' U' F'".into()),
            result_case_number: Some(21),
            result_rotation: Some(1),
            tags: vec!["fish".into(), "needs work".into()],
        },
    );

    import(&db.pool, user, &state).await.unwrap();

    let row: (Option<String>, Option<String>, Option<i32>, Option<Vec<String>>) =
        sqlx::query_as(
            "SELECT s.nickname, s.algorithm, s.result_rotation, s.tags \
             FROM user_case_settings s \
             JOIN cases c ON c.id = s.case_id \
             WHERE s.user_id = $1 AND c.case_number = 12",
        )
        .bind(user)
        .fetch_one(&db.pool)
        .await
        .expect("settings row exists");

    assert_eq!(row.0.as_deref(), Some("Slash"));
    assert_eq!(row.1.as_deref(), Some("F R U R' U' F'"));
    assert_eq!(row.2, Some(1));
    assert_eq!(row.3, Some(vec!["fish".into(), "needs work".into()]));

    // result_case_number 21 should have translated to a real case_id.
    let result_case_number: (Option<i32>,) = sqlx::query_as(
        "SELECT rc.case_number FROM user_case_settings s \
         JOIN cases c ON c.id = s.case_id \
         LEFT JOIN cases rc ON rc.id = s.result_case_id \
         WHERE s.user_id = $1 AND c.case_number = 12",
    )
    .bind(user)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(result_case_number.0, Some(21));
}

#[tokio::test]
async fn import_progress_lands_with_sm2_fields() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let mut state = empty_state();
    state.progress.insert(
        "5".into(),
        GuestProgress {
            ease_factor: 2.7,
            interval_days: 6,
            repetitions: 2,
            due_date: NaiveDate::from_ymd_opt(2026, 6, 1).unwrap(),
            last_grade: Some(2),
            last_reviewed: Some(Utc::now()),
        },
    );

    import(&db.pool, user, &state).await.unwrap();

    let row: (f64, i32, i32, NaiveDate, Option<i32>) = sqlx::query_as(
        "SELECT p.ease_factor, p.interval_days, p.repetitions, p.due_date, p.last_grade \
         FROM user_case_progress p \
         JOIN cases c ON c.id = p.case_id \
         WHERE p.user_id = $1 AND c.case_number = 5",
    )
    .bind(user)
    .fetch_one(&db.pool)
    .await
    .expect("progress row exists");

    assert!((row.0 - 2.7).abs() < 1e-9);
    assert_eq!(row.1, 6);
    assert_eq!(row.2, 2);
    assert_eq!(row.3, NaiveDate::from_ymd_opt(2026, 6, 1).unwrap());
    assert_eq!(row.4, Some(2));
}

#[tokio::test]
async fn import_streak_and_onboarding_propagate() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let mut state = empty_state();
    state.streak_count = 7;
    state.last_practice_date = NaiveDate::from_ymd_opt(2026, 5, 4);
    state.onboarding_completed = true;

    import(&db.pool, user, &state).await.unwrap();

    let row: (i32, Option<NaiveDate>, bool) = sqlx::query_as(
        "SELECT streak_count, last_practice_date, has_seen_onboarding FROM users WHERE id = $1",
    )
    .bind(user)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(row.0, 7);
    assert_eq!(row.1, NaiveDate::from_ymd_opt(2026, 5, 4));
    assert!(row.2);
}

#[tokio::test]
async fn import_does_not_clear_existing_onboarding_when_blob_says_false() {
    // Edge case: user is already authed (has_seen_onboarding = TRUE), then
    // imports a blob from a guest session that never finished onboarding.
    // The import path must not regress the flag — it ORs the values.
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    sqlx::query("UPDATE users SET has_seen_onboarding = TRUE WHERE id = $1")
        .bind(user)
        .execute(&db.pool)
        .await
        .unwrap();

    let state = empty_state(); // onboarding_completed = false

    import(&db.pool, user, &state).await.unwrap();

    let row: (bool,) =
        sqlx::query_as("SELECT has_seen_onboarding FROM users WHERE id = $1")
            .bind(user)
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert!(row.0, "must not regress to false");
}

#[tokio::test]
async fn import_skips_settings_rows_with_no_real_overrides() {
    // A blob where a settings entry has all-None fields shouldn't create
    // an empty row — it'd just be wasted space.
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let mut state = empty_state();
    state.settings.insert(
        "1".into(),
        GuestSettings {
            nickname: None,
            algorithm: None,
            result_case_number: None,
            result_rotation: None,
            tags: vec![],
        },
    );
    state.settings.insert(
        "2".into(),
        GuestSettings {
            nickname: Some("Real Override".into()),
            algorithm: None,
            result_case_number: None,
            result_rotation: None,
            tags: vec![],
        },
    );

    import(&db.pool, user, &state).await.unwrap();

    let count: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM user_case_settings WHERE user_id = $1",
    )
    .bind(user)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(count.0, 1, "only the real override should land");
}

// ─── Tag caps ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn import_rejects_blob_over_distinct_tag_cap() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let mut state = empty_state();
    let tags: Vec<String> = (0..=MAX_DISTINCT_TAGS_PER_USER)
        .map(|i| format!("t{i}"))
        .collect();
    state.settings.insert(
        "1".into(),
        GuestSettings {
            nickname: None,
            algorithm: None,
            result_case_number: None,
            result_rotation: None,
            tags,
        },
    );

    let err = import(&db.pool, user, &state).await.expect_err("over cap rejects");
    match err {
        AppError::Validation(_) => {}
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[tokio::test]
async fn import_rejects_blob_over_link_cap() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let mut state = empty_state();
    // 50 cases × 21 tags each = 1050 links — over the 1000 cap.
    for n in 1..=50i32 {
        let tags: Vec<String> = (0..21).map(|i| format!("p{i}")).collect();
        state.settings.insert(
            n.to_string(),
            GuestSettings {
                nickname: None,
                algorithm: None,
                result_case_number: None,
                result_rotation: None,
                tags,
            },
        );
    }

    let err = import(&db.pool, user, &state).await.expect_err("over link cap rejects");
    match err {
        AppError::Validation(_) => {}
        other => panic!("expected Validation, got {other:?}"),
    }

    // Cap-trip should have rolled back — no settings rows for this user.
    let count: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM user_case_settings WHERE user_id = $1",
    )
    .bind(user)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    let _ = MAX_TAG_LINKS_PER_USER; // silence unused warning if we ever drop the assertion
    assert_eq!(count.0, 0, "import must roll back on cap trip");
}

// ─── Validation gate ─────────────────────────────────────────────────────────

#[tokio::test]
async fn validate_catches_bad_blob_before_db_touched() {
    // The route handler runs validate() before opening the transaction.
    // This test guards that contract — a known-bad blob fails validate
    // and is never handed to import().
    let mut state = empty_state();
    state.version = 99; // wrong version

    state.validate().expect_err("bad version rejects");
}

// ─── Defense-in-depth — import() called without validate() first ─────────────
//
// validate() is the contract gate, but `import` is `pub` and a future caller
// could forget. These tests pin down the structured errors `import` returns
// when fed malformed/inconsistent input, so the registration handler always
// surfaces a 400 with the right field instead of leaking a 500.

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
async fn import_rejects_non_numeric_settings_key() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let mut state = empty_state();
    state.settings.insert("not-a-number".into(), empty_settings());

    let err = import(&db.pool, user, &state).await.expect_err("rejects");
    match err {
        AppError::Validation(fields) => {
            assert!(fields.contains_key("guest_state.settings"), "{fields:?}");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[tokio::test]
async fn import_rejects_non_numeric_progress_key() {
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

    let err = import(&db.pool, user, &state).await.expect_err("rejects");
    match err {
        AppError::Validation(fields) => {
            assert!(fields.contains_key("guest_state.progress"), "{fields:?}");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

/// Drop the FK-back-references from cases.result_case_id and remove a single
/// case row. Lets us simulate a degraded DB state where a guest blob keyed
/// to a valid case_number maps to nothing in `cases`.
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
async fn import_rejects_settings_pointing_at_missing_case() {
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

    let err = import(&db.pool, user, &state).await.expect_err("rejects");
    match err {
        AppError::Validation(fields) => {
            assert!(fields.contains_key("guest_state.settings"), "{fields:?}");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[tokio::test]
async fn import_rejects_settings_with_missing_result_case() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    delete_case(&db.pool, 30).await;

    // Settings on case 1, but it points at a deleted case 30 as its
    // result. Hits the result_case_number lookup branch.
    let mut state = empty_state();
    state.settings.insert(
        "1".into(),
        GuestSettings {
            result_case_number: Some(30),
            ..empty_settings()
        },
    );

    let err = import(&db.pool, user, &state).await.expect_err("rejects");
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
async fn import_rejects_progress_pointing_at_missing_case() {
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

    let err = import(&db.pool, user, &state).await.expect_err("rejects");
    match err {
        AppError::Validation(fields) => {
            assert!(fields.contains_key("guest_state.progress"), "{fields:?}");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}
