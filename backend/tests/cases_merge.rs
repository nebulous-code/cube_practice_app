//! Integration tests for the cases override-merge logic powering
//! `GET /cases` (and later `GET /cases/:id`).

mod common;

use common::TestDb;
use cube_backend::cases;
use uuid::Uuid;

#[tokio::test]
async fn lists_all_57_cases_with_no_overrides() {
    let db = TestDb::new().await;
    let user_id = seed_user(&db.pool, "alice@example.com").await;

    let list = cases::list_for_user(&db.pool, user_id)
        .await
        .expect("list");

    assert_eq!(list.len(), 57);
    for case in &list {
        assert_eq!(case.solve_stage, "OLL");
        assert_eq!(case.puzzle_type, "3x3");
        assert!(!case.has_overrides);
        assert_eq!(case.pattern.len(), 9, "pattern should be 9 chars");
    }

    // Sorted by case_number ASC.
    let numbers: Vec<i32> = list.iter().map(|c| c.case_number).collect();
    let mut sorted = numbers.clone();
    sorted.sort();
    assert_eq!(numbers, sorted);
}

#[tokio::test]
async fn applies_nickname_override() {
    let db = TestDb::new().await;
    let user_id = seed_user(&db.pool, "bob@example.com").await;

    let case_id = case_id_by_number(&db.pool, 1).await;

    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, nickname) VALUES ($1, $2, 'My Sune')",
    )
    .bind(user_id)
    .bind(case_id)
    .execute(&db.pool)
    .await
    .expect("insert override");

    let case = cases::get_for_user(&db.pool, user_id, case_id)
        .await
        .expect("get");
    assert_eq!(case.nickname.as_deref(), Some("My Sune"));
    assert!(case.has_overrides);

    // Other fields fall through to defaults.
    assert_eq!(case.case_number, 1);
    assert_eq!(case.tier1_tag, "*");
    assert_eq!(case.tier2_tag.as_deref(), Some("dot"));
}

#[tokio::test]
async fn null_override_falls_through_to_default() {
    let db = TestDb::new().await;
    let user_id = seed_user(&db.pool, "carol@example.com").await;
    let case_id = case_id_by_number(&db.pool, 1).await;

    // Insert an override row that touches algorithm only — nickname stays NULL
    // and should fall through to the global "Tie Fighter".
    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, algorithm) VALUES ($1, $2, 'CUSTOM')",
    )
    .bind(user_id)
    .bind(case_id)
    .execute(&db.pool)
    .await
    .expect("insert override");

    let case = cases::get_for_user(&db.pool, user_id, case_id)
        .await
        .expect("get");
    assert_eq!(case.nickname.as_deref(), Some("Tie Fighter"));
    assert_eq!(case.algorithm, "CUSTOM");
    assert!(case.has_overrides);
}

#[tokio::test]
async fn result_case_id_override_changes_result_case_number() {
    let db = TestDb::new().await;
    let user_id = seed_user(&db.pool, "dave@example.com").await;

    let case_1 = case_id_by_number(&db.pool, 1).await;
    let case_30 = case_id_by_number(&db.pool, 30).await;

    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, result_case_id, result_rotation) \
         VALUES ($1, $2, $3, 1)",
    )
    .bind(user_id)
    .bind(case_1)
    .bind(case_30)
    .execute(&db.pool)
    .await
    .expect("insert override");

    let case = cases::get_for_user(&db.pool, user_id, case_1)
        .await
        .expect("get");
    assert_eq!(case.result_case_id, Some(case_30));
    assert_eq!(case.result_case_number, Some(30));
    assert_eq!(case.result_rotation, 1);
}

#[tokio::test]
async fn other_users_overrides_dont_leak() {
    let db = TestDb::new().await;
    let alice = seed_user(&db.pool, "alice@example.com").await;
    let bob = seed_user(&db.pool, "bob@example.com").await;
    let case_id = case_id_by_number(&db.pool, 1).await;

    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, nickname) VALUES ($1, $2, 'Alices')",
    )
    .bind(alice)
    .bind(case_id)
    .execute(&db.pool)
    .await
    .expect("insert");

    let bobs_case = cases::get_for_user(&db.pool, bob, case_id)
        .await
        .expect("get");
    assert_eq!(bobs_case.nickname.as_deref(), Some("Tie Fighter"));
    assert!(!bobs_case.has_overrides);

    let alices_case = cases::get_for_user(&db.pool, alice, case_id)
        .await
        .expect("get");
    assert_eq!(alices_case.nickname.as_deref(), Some("Alices"));
    assert!(alices_case.has_overrides);
}

#[tokio::test]
async fn get_for_user_returns_not_found_for_unknown_id() {
    let db = TestDb::new().await;
    let user_id = seed_user(&db.pool, "alice@example.com").await;

    let err = cases::get_for_user(&db.pool, user_id, Uuid::new_v4())
        .await
        .expect_err("should be not found");
    assert!(matches!(err, cube_backend::error::AppError::NotFound));
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
    .expect("insert user");
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
    .expect("lookup case");
    row.0
}
