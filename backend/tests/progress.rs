//! Integration tests for the progress module — `summary_for_user` (state
//! count breakdown + streak) and `cases_for_user` (filtered list).

mod common;

use chrono::{Duration, NaiveDate, Utc};
use common::TestDb;
use cube_backend::cases::CaseState;
use cube_backend::progress;
use uuid::Uuid;

fn today() -> NaiveDate {
    Utc::now().date_naive()
}

// ─── summary_for_user ────────────────────────────────────────────────────────

#[tokio::test]
async fn summary_fresh_user_has_all_57_not_started() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let response = progress::summary_for_user(&db.pool, user).await.unwrap();
    assert_eq!(response.total, 57);
    assert_eq!(response.summary.not_started, 57);
    assert_eq!(response.summary.learning, 0);
    assert_eq!(response.summary.due, 0);
    assert_eq!(response.summary.mastered, 0);
    assert_eq!(response.streak.count, 0);
    assert!(response.streak.last_practice_date.is_none());
}

#[tokio::test]
async fn summary_mixed_states_sum_to_total() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    // Case 1 due, case 2 learning, case 3 mastered.
    insert_progress(&db.pool, user, case_id(&db.pool, 1).await, today() - Duration::days(1), 1).await;
    insert_progress(&db.pool, user, case_id(&db.pool, 2).await, today() + Duration::days(5), 5).await;
    insert_progress(&db.pool, user, case_id(&db.pool, 3).await, today() + Duration::days(30), 30).await;

    let response = progress::summary_for_user(&db.pool, user).await.unwrap();
    assert_eq!(response.total, 57);
    assert_eq!(response.summary.due, 1);
    assert_eq!(response.summary.learning, 1);
    assert_eq!(response.summary.mastered, 1);
    assert_eq!(response.summary.not_started, 54);
    assert_eq!(
        response.summary.not_started
            + response.summary.learning
            + response.summary.due
            + response.summary.mastered,
        response.total,
    );
}

#[tokio::test]
async fn summary_isolates_users() {
    let db = TestDb::new().await;
    let alice = seed_user(&db.pool, "alice@example.com").await;
    let bob = seed_user(&db.pool, "bob@example.com").await;

    insert_progress(&db.pool, alice, case_id(&db.pool, 1).await, today() - Duration::days(1), 1).await;

    let bobs = progress::summary_for_user(&db.pool, bob).await.unwrap();
    assert_eq!(bobs.summary.not_started, 57);
    assert_eq!(bobs.summary.due, 0);
}

// ─── cases_for_user ──────────────────────────────────────────────────────────

#[tokio::test]
async fn cases_no_filter_returns_all() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let cases = progress::cases_for_user(&db.pool, user, None).await.unwrap();
    assert_eq!(cases.len(), 57);
}

#[tokio::test]
async fn cases_filter_not_started_excludes_progress_rows() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let case_1 = case_id(&db.pool, 1).await;
    insert_progress(&db.pool, user, case_1, today() - Duration::days(1), 1).await;

    let cases = progress::cases_for_user(&db.pool, user, Some(CaseState::NotStarted))
        .await
        .unwrap();
    assert_eq!(cases.len(), 56);
    assert!(cases.iter().all(|c| c.id != case_1));
    assert!(cases.iter().all(|c| c.state == CaseState::NotStarted));
}

#[tokio::test]
async fn cases_filter_due() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let case_1 = case_id(&db.pool, 1).await;
    let case_2 = case_id(&db.pool, 2).await;
    insert_progress(&db.pool, user, case_1, today() - Duration::days(1), 1).await;
    insert_progress(&db.pool, user, case_2, today() + Duration::days(5), 5).await; // learning

    let cases = progress::cases_for_user(&db.pool, user, Some(CaseState::Due))
        .await
        .unwrap();
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].id, case_1);
    assert_eq!(cases[0].state, CaseState::Due);
}

#[tokio::test]
async fn cases_filter_learning() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let case_1 = case_id(&db.pool, 1).await;
    insert_progress(&db.pool, user, case_1, today() + Duration::days(5), 5).await;

    let cases = progress::cases_for_user(&db.pool, user, Some(CaseState::Learning))
        .await
        .unwrap();
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].id, case_1);
    assert_eq!(cases[0].state, CaseState::Learning);
}

#[tokio::test]
async fn cases_filter_mastered() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let case_1 = case_id(&db.pool, 1).await;
    insert_progress(&db.pool, user, case_1, today() + Duration::days(30), 30).await;

    let cases = progress::cases_for_user(&db.pool, user, Some(CaseState::Mastered))
        .await
        .unwrap();
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].id, case_1);
    assert_eq!(cases[0].state, CaseState::Mastered);
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

async fn case_id(pool: &sqlx::PgPool, n: i32) -> Uuid {
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

async fn insert_progress(
    pool: &sqlx::PgPool,
    user: Uuid,
    case: Uuid,
    due: NaiveDate,
    interval_days: i32,
) {
    sqlx::query(
        "INSERT INTO user_case_progress (user_id, case_id, due_date, interval_days, repetitions) \
         VALUES ($1, $2, $3, $4, 1)",
    )
    .bind(user)
    .bind(case)
    .bind(due)
    .bind(interval_days)
    .execute(pool)
    .await
    .unwrap();
}
