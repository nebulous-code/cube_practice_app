//! Schema-level checks for the M3 `user_case_progress` migration. Verifies
//! the table exists with the expected columns and that CHECK / FK / UNIQUE
//! constraints reject invalid data.

mod common;

use common::TestDb;
use uuid::Uuid;

#[tokio::test]
async fn table_exists() {
    let db = TestDb::new().await;
    let row: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM information_schema.tables \
         WHERE table_schema = 'public' AND table_name = 'user_case_progress'",
    )
    .fetch_one(&db.pool)
    .await
    .expect("info schema");
    assert_eq!(row.0, 1);
}

#[tokio::test]
async fn rejects_negative_interval() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    let err = sqlx::query(
        "INSERT INTO user_case_progress (user_id, case_id, interval_days) VALUES ($1, $2, 0)",
    )
    .bind(user)
    .bind(case)
    .execute(&db.pool)
    .await
    .expect_err("interval_days check");

    assert!(format!("{err}").contains("check"));
}

#[tokio::test]
async fn rejects_negative_repetitions() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    let err = sqlx::query(
        "INSERT INTO user_case_progress (user_id, case_id, repetitions) VALUES ($1, $2, -1)",
    )
    .bind(user)
    .bind(case)
    .execute(&db.pool)
    .await
    .expect_err("repetitions check");

    assert!(format!("{err}").contains("check"));
}

#[tokio::test]
async fn rejects_grade_out_of_range() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    let err = sqlx::query(
        "INSERT INTO user_case_progress (user_id, case_id, last_grade) VALUES ($1, $2, 4)",
    )
    .bind(user)
    .bind(case)
    .execute(&db.pool)
    .await
    .expect_err("last_grade check");

    assert!(format!("{err}").contains("check"));
}

#[tokio::test]
async fn unique_user_case_pair() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    sqlx::query("INSERT INTO user_case_progress (user_id, case_id) VALUES ($1, $2)")
        .bind(user)
        .bind(case)
        .execute(&db.pool)
        .await
        .expect("first insert");

    let err = sqlx::query("INSERT INTO user_case_progress (user_id, case_id) VALUES ($1, $2)")
        .bind(user)
        .bind(case)
        .execute(&db.pool)
        .await
        .expect_err("duplicate (user, case) should fail");

    assert!(format!("{err}").contains("unique") || format!("{err}").contains("23505"));
}

#[tokio::test]
async fn cascades_on_user_delete() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    sqlx::query("INSERT INTO user_case_progress (user_id, case_id) VALUES ($1, $2)")
        .bind(user)
        .bind(case)
        .execute(&db.pool)
        .await
        .expect("insert");

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user)
        .execute(&db.pool)
        .await
        .expect("delete user");

    let count: (i64,) = sqlx::query_as(
        "SELECT count(*)::bigint FROM user_case_progress WHERE user_id = $1",
    )
    .bind(user)
    .fetch_one(&db.pool)
    .await
    .expect("count");
    assert_eq!(count.0, 0);
}

#[tokio::test]
async fn defaults_are_anki_initials() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let case = case_id_by_number(&db.pool, 1).await;

    sqlx::query("INSERT INTO user_case_progress (user_id, case_id) VALUES ($1, $2)")
        .bind(user)
        .bind(case)
        .execute(&db.pool)
        .await
        .unwrap();

    let row: (f64, i32, i32) = sqlx::query_as(
        "SELECT ease_factor, interval_days, repetitions FROM user_case_progress \
         WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user)
    .bind(case)
    .fetch_one(&db.pool)
    .await
    .unwrap();

    assert_eq!(row.0, 2.5);
    assert_eq!(row.1, 1);
    assert_eq!(row.2, 0);
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
