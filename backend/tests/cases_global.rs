//! M6 B1 — anonymous reads of the case set return globals only, never
//! leak per-user overrides or progress. See docs/milestones/06_guest_mode.md
//! §4 + §6.

mod common;

use chrono::{Duration, Utc};
use common::TestDb;
use cube_backend::cases::{self, CaseState};
use uuid::Uuid;

async fn seed_user(pool: &sqlx::PgPool, email: &str) -> Uuid {
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

async fn case_id_for(pool: &sqlx::PgPool, n: i32) -> Uuid {
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM cases WHERE case_number = $1")
        .bind(n)
        .fetch_one(pool)
        .await
        .expect("case lookup");
    row.0
}

#[tokio::test]
async fn list_global_returns_all_57_cases() {
    let db = TestDb::new().await;
    let cases = cases::list_global(&db.pool).await.unwrap();
    assert_eq!(cases.len(), 57);
}

#[tokio::test]
async fn list_global_returns_not_started_state_for_every_case() {
    let db = TestDb::new().await;
    let cases = cases::list_global(&db.pool).await.unwrap();
    for c in cases {
        assert_eq!(c.state, CaseState::NotStarted, "case {} should be not_started", c.case_number);
    }
}

#[tokio::test]
async fn list_global_has_no_overrides_set() {
    let db = TestDb::new().await;
    let cases = cases::list_global(&db.pool).await.unwrap();
    assert!(cases.iter().all(|c| !c.has_overrides));
}

#[tokio::test]
async fn list_global_does_not_leak_user_overrides_or_progress() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_for(&db.pool, 12).await;

    // Insert a per-user override + a learning-state progress row for case 12.
    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, nickname) VALUES ($1, $2, 'Slash')",
    )
    .bind(user)
    .bind(case)
    .execute(&db.pool)
    .await
    .expect("insert override");

    sqlx::query(
        "INSERT INTO user_case_progress (user_id, case_id, due_date, interval_days, ease_factor, repetitions, last_grade) \
         VALUES ($1, $2, $3, 6, 2.5, 2, 2)",
    )
    .bind(user)
    .bind(case)
    .bind((Utc::now() + Duration::days(6)).date_naive())
    .execute(&db.pool)
    .await
    .expect("insert progress");

    let global_view = cases::get_global(&db.pool, case).await.unwrap();
    assert!(!global_view.has_overrides, "global view must not surface alice's override");
    assert_ne!(
        global_view.nickname.as_deref(),
        Some("Slash"),
        "global nickname must not match alice's override"
    );
    assert_eq!(
        global_view.state,
        CaseState::NotStarted,
        "global state must not surface alice's progress"
    );
}

#[tokio::test]
async fn get_global_unknown_uuid_returns_not_found() {
    let db = TestDb::new().await;
    let err = cases::get_global(&db.pool, Uuid::new_v4()).await.unwrap_err();
    match err {
        cube_backend::error::AppError::NotFound => {}
        other => panic!("expected NotFound, got {other:?}"),
    }
}

#[tokio::test]
async fn list_for_user_still_merges_overrides_after_b1() {
    // Regression: B1 changed the route layer to use Option<AuthUser>; the
    // authed path must still merge overrides via list_for_user.
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_for(&db.pool, 7).await;

    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, nickname) VALUES ($1, $2, 'My Sune')",
    )
    .bind(user)
    .bind(case)
    .execute(&db.pool)
    .await
    .expect("insert override");

    let alices_view = cases::get_for_user(&db.pool, user, case).await.unwrap();
    assert_eq!(alices_view.nickname.as_deref(), Some("My Sune"));
    assert!(alices_view.has_overrides);
}
