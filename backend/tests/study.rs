//! Integration tests for the study module — `due_for_user` filtering and
//! `apply_review` SM-2/streak update behavior.

mod common;

use chrono::{Duration, NaiveDate, Utc};
use common::TestDb;
use cube_backend::cases::CaseState;
use cube_backend::error::AppError;
use cube_backend::srs::Grade;
use cube_backend::study;
use uuid::Uuid;

fn today() -> NaiveDate {
    Utc::now().date_naive()
}

// ─── due_for_user ────────────────────────────────────────────────────────────

#[tokio::test]
async fn due_returns_empty_for_fresh_user() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let response = study::due_for_user(&db.pool, user).await.unwrap();
    assert_eq!(response.cases.len(), 0);
    assert_eq!(response.streak.count, 0);
    assert!(response.streak.last_practice_date.is_none());
}

#[tokio::test]
async fn due_includes_only_due_state_cases() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let case_due = case_id_by_number(&db.pool, 1).await;
    let case_learning = case_id_by_number(&db.pool, 2).await;

    // case 1: due yesterday
    insert_progress(&db.pool, user, case_due, today() - Duration::days(1), 1).await;
    // case 2: due in 5 days, interval 5 → learning, not due
    insert_progress(&db.pool, user, case_learning, today() + Duration::days(5), 5).await;

    let response = study::due_for_user(&db.pool, user).await.unwrap();
    assert_eq!(response.cases.len(), 1);
    assert_eq!(response.cases[0].id, case_due);
    assert_eq!(response.cases[0].state, CaseState::Due);
}

#[tokio::test]
async fn due_sorts_oldest_first() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let case_a = case_id_by_number(&db.pool, 1).await;
    let case_b = case_id_by_number(&db.pool, 2).await;

    insert_progress(&db.pool, user, case_a, today() - Duration::days(2), 1).await;
    insert_progress(&db.pool, user, case_b, today() - Duration::days(5), 1).await;

    let response = study::due_for_user(&db.pool, user).await.unwrap();
    assert_eq!(response.cases[0].id, case_b, "older due should be first");
    assert_eq!(response.cases[1].id, case_a);
}

#[tokio::test]
async fn due_does_not_leak_across_users() {
    let db = TestDb::new().await;
    let alice = seed_user(&db.pool, "alice@example.com").await;
    let bob = seed_user(&db.pool, "bob@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    insert_progress(&db.pool, alice, case, today() - Duration::days(1), 1).await;

    let bobs = study::due_for_user(&db.pool, bob).await.unwrap();
    assert_eq!(bobs.cases.len(), 0);

    let alices = study::due_for_user(&db.pool, alice).await.unwrap();
    assert_eq!(alices.cases.len(), 1);
}

// ─── apply_review ────────────────────────────────────────────────────────────

#[tokio::test]
async fn first_review_creates_progress_row() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    let response = study::apply_review(&db.pool, user, case, Grade::Good, today())
        .await
        .unwrap();

    // After Good at rep 0: rep=1, interval=1, due=tomorrow → state=learning.
    assert_eq!(response.case.state, CaseState::Learning);
    assert_eq!(response.streak.count, 1);
    assert_eq!(response.streak.last_practice_date, Some(today()));

    let count: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM user_case_progress WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user)
    .bind(case)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(count.0, 1);
}

#[tokio::test]
async fn second_review_updates_existing_row() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    study::apply_review(&db.pool, user, case, Grade::Good, today())
        .await
        .unwrap();
    study::apply_review(&db.pool, user, case, Grade::Good, today())
        .await
        .unwrap();

    let count: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM user_case_progress WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user)
    .bind(case)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(count.0, 1);
}

#[tokio::test]
async fn streak_holds_at_one_for_two_reviews_same_day() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case_a = case_id_by_number(&db.pool, 1).await;
    let case_b = case_id_by_number(&db.pool, 2).await;

    let r1 = study::apply_review(&db.pool, user, case_a, Grade::Good, today())
        .await
        .unwrap();
    assert_eq!(r1.streak.count, 1);

    let r2 = study::apply_review(&db.pool, user, case_b, Grade::Good, today())
        .await
        .unwrap();
    assert_eq!(r2.streak.count, 1, "same-day reviews don't increment");
}

#[tokio::test]
async fn streak_ticks_when_last_practice_was_yesterday() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    let yesterday = today() - Duration::days(1);
    sqlx::query(
        "UPDATE users SET streak_count = 4, last_practice_date = $2 WHERE id = $1",
    )
    .bind(user)
    .bind(yesterday)
    .execute(&db.pool)
    .await
    .unwrap();

    let response = study::apply_review(&db.pool, user, case, Grade::Good, today())
        .await
        .unwrap();
    assert_eq!(response.streak.count, 5);
}

#[tokio::test]
async fn streak_resets_when_gap_is_two_days() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    let two_days_ago = today() - Duration::days(2);
    sqlx::query(
        "UPDATE users SET streak_count = 10, last_practice_date = $2 WHERE id = $1",
    )
    .bind(user)
    .bind(two_days_ago)
    .execute(&db.pool)
    .await
    .unwrap();

    let response = study::apply_review(&db.pool, user, case, Grade::Good, today())
        .await
        .unwrap();
    assert_eq!(response.streak.count, 1);
}

#[tokio::test]
async fn unknown_case_returns_not_found() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    let err = study::apply_review(&db.pool, user, Uuid::new_v4(), Grade::Good, today())
        .await
        .expect_err("not found");
    assert!(matches!(err, AppError::NotFound));
}

#[tokio::test]
async fn fail_after_streak_resets_card_progress() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    // Build up some progress: 3 Goods.
    let mut t = today() - Duration::days(20);
    for _ in 0..3 {
        let _ = study::apply_review(&db.pool, user, case, Grade::Good, t).await;
        t += Duration::days(7);
    }

    let after_fail = study::apply_review(&db.pool, user, case, Grade::Fail, today())
        .await
        .unwrap();
    // Fail reschedules to tomorrow (interval=1 from today), so the card is
    // not currently `due` — it's `learning` until tomorrow rolls over.
    assert_eq!(after_fail.case.state, CaseState::Learning);

    let row: (f64, i32, i32, NaiveDate) = sqlx::query_as(
        "SELECT ease_factor, interval_days, repetitions, due_date FROM user_case_progress \
         WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user)
    .bind(case)
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(row.1, 1, "fail resets interval to 1");
    assert_eq!(row.2, 0, "fail resets repetitions");
    assert!(row.0 < 2.5, "fail drops ease");
    assert_eq!(row.3, today() + Duration::days(1), "due tomorrow after fail");
}

#[tokio::test]
async fn reviews_dont_leak_across_users() {
    let db = TestDb::new().await;
    let alice = seed_user(&db.pool, "alice@example.com").await;
    let bob = seed_user(&db.pool, "bob@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    study::apply_review(&db.pool, alice, case, Grade::Good, today())
        .await
        .unwrap();

    let bobs_due = study::due_for_user(&db.pool, bob).await.unwrap();
    assert_eq!(bobs_due.cases.len(), 0);
    assert_eq!(bobs_due.streak.count, 0);
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
