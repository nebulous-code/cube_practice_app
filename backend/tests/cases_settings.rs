//! Integration tests for `cases::update_settings` (PATCH endpoint logic).
//! Covers the Option<Option<T>> patch semantics, same-stage validation,
//! all-null cleanup, and cross-user isolation.

mod common;

use common::TestDb;
use cube_backend::cases::{self, SettingsPatch};
use cube_backend::error::AppError;
use uuid::Uuid;

#[tokio::test]
async fn setting_a_nickname_creates_override_row() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    let patch = SettingsPatch {
        nickname: Some(Some("My Sune".into())),
        ..Default::default()
    };
    let merged = cases::update_settings(&db.pool, user, case, patch)
        .await
        .expect("update");

    assert_eq!(merged.nickname.as_deref(), Some("My Sune"));
    assert!(merged.has_overrides);

    // Other fields fall through.
    assert_eq!(merged.tier1_tag, "*");
}

#[tokio::test]
async fn null_clears_an_existing_override() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    cases::update_settings(
        &db.pool,
        user,
        case,
        SettingsPatch {
            nickname: Some(Some("Mine".into())),
            algorithm: Some(Some("R U".into())),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Now clear nickname only — algorithm stays.
    let merged = cases::update_settings(
        &db.pool,
        user,
        case,
        SettingsPatch {
            nickname: Some(None),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(merged.nickname.as_deref(), Some("Tie Fighter"));
    assert_eq!(merged.algorithm, "R U");
    assert!(merged.has_overrides);
}

#[tokio::test]
async fn clearing_all_fields_deletes_the_row() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    cases::update_settings(
        &db.pool,
        user,
        case,
        SettingsPatch {
            nickname: Some(Some("Mine".into())),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let merged = cases::update_settings(
        &db.pool,
        user,
        case,
        SettingsPatch {
            nickname: Some(None),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert!(!merged.has_overrides);
    assert_eq!(merged.nickname.as_deref(), Some("Tie Fighter"));

    let count: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM user_case_settings WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user)
    .bind(case)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(count.0, 0, "all-null upsert should delete the row");
}

#[tokio::test]
async fn absent_field_leaves_existing_override_untouched() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    cases::update_settings(
        &db.pool,
        user,
        case,
        SettingsPatch {
            nickname: Some(Some("Mine".into())),
            algorithm: Some(Some("R U R'".into())),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Patch only nickname — algorithm should stay "R U R'".
    let merged = cases::update_settings(
        &db.pool,
        user,
        case,
        SettingsPatch {
            nickname: Some(Some("Renamed".into())),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(merged.nickname.as_deref(), Some("Renamed"));
    assert_eq!(merged.algorithm, "R U R'");
}

#[tokio::test]
async fn result_case_id_must_be_in_same_stage() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    // Insert a fake second stage + case to exercise the cross-stage path.
    let pt: (Uuid,) =
        sqlx::query_as("INSERT INTO puzzle_types (name) VALUES ('alt') RETURNING id")
            .fetch_one(&db.pool)
            .await
            .unwrap();
    let stage: (Uuid,) = sqlx::query_as(
        "INSERT INTO solve_stages (puzzle_type_id, name) VALUES ($1, 'PLL') RETURNING id",
    )
    .bind(pt.0)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    let other_case: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO cases
          (solve_stage_id, case_number, algorithm, diagram_data, tier1_tag)
        VALUES ($1, 1, 'R', '{"pattern":"X"}'::jsonb, '*')
        RETURNING id
        "#,
    )
    .bind(stage.0)
    .fetch_one(&db.pool)
    .await
    .unwrap();

    let err = cases::update_settings(
        &db.pool,
        user,
        case,
        SettingsPatch {
            result_case_id: Some(Some(other_case.0)),
            ..Default::default()
        },
    )
    .await
    .expect_err("should reject cross-stage result");

    match err {
        AppError::Validation(fields) => {
            assert!(fields.contains_key("result_case_id"));
        }
        _ => panic!("expected Validation, got {err:?}"),
    }
}

#[tokio::test]
async fn result_case_id_unknown_uuid_rejected() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    let err = cases::update_settings(
        &db.pool,
        user,
        case,
        SettingsPatch {
            result_case_id: Some(Some(Uuid::new_v4())),
            ..Default::default()
        },
    )
    .await
    .expect_err("should reject unknown result_case_id");

    matches!(err, AppError::Validation(_));
}

#[tokio::test]
async fn unknown_case_id_returns_not_found() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let err = cases::update_settings(
        &db.pool,
        user,
        Uuid::new_v4(),
        SettingsPatch::default(),
    )
    .await
    .expect_err("not found");
    assert!(matches!(err, AppError::NotFound));
}

#[tokio::test]
async fn updates_dont_leak_across_users() {
    let db = TestDb::new().await;
    let alice = seed_user(&db.pool, "alice@example.com").await;
    let bob = seed_user(&db.pool, "bob@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    cases::update_settings(
        &db.pool,
        alice,
        case,
        SettingsPatch {
            nickname: Some(Some("Alices".into())),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    let bobs = cases::get_for_user(&db.pool, bob, case).await.unwrap();
    assert_eq!(bobs.nickname.as_deref(), Some("Tie Fighter"));
    assert!(!bobs.has_overrides);
}

#[tokio::test]
async fn tags_override_replaces_global() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    let merged = cases::update_settings(
        &db.pool,
        user,
        case,
        SettingsPatch {
            tags: Some(Some(vec!["fish".into(), "needs work".into()])),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    assert_eq!(merged.tags, vec!["fish", "needs work"]);
    assert!(merged.has_overrides);
}

#[tokio::test]
async fn tags_some_none_clears_to_global() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    // First set an override.
    cases::update_settings(
        &db.pool,
        user,
        case,
        SettingsPatch {
            tags: Some(Some(vec!["fish".into()])),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Now clear it.
    let merged = cases::update_settings(
        &db.pool,
        user,
        case,
        SettingsPatch {
            tags: Some(None),
            ..Default::default()
        },
    )
    .await
    .unwrap();

    // Falls back to the seeded global tag for case 01 ("dot").
    assert_eq!(merged.tags, vec!["dot"]);
    // No other override fields, so the row should be deleted.
    assert!(!merged.has_overrides);
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

async fn seed_user(pool: &sqlx::PgPool, email: &str) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "INSERT INTO users (email, display_name, password_hash, email_verified) \
         VALUES ($1, 'Test', 'x', TRUE) RETURNING id",
    )
    .bind(email)
    .fetch_one(pool)
    .await
    .unwrap();
    row.0
}

async fn case_id_by_number(pool: &sqlx::PgPool, n: i32) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        "SELECT c.id FROM cases c \
         JOIN solve_stages s ON s.id = c.solve_stage_id AND s.name = 'OLL' \
         WHERE c.case_number = $1",
    )
    .bind(n)
    .fetch_one(pool)
    .await
    .unwrap();
    row.0
}
