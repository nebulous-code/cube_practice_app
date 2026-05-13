//! Schema-level checks for the M2 `cases` migration. Verifies the tables
//! exist with the expected columns and that CHECK / FK / UNIQUE constraints
//! reject invalid data.

mod common;

use common::TestDb;
use sqlx::Row;
use uuid::Uuid;

#[tokio::test]
async fn tables_exist() {
    let db = TestDb::new().await;

    let expected = ["puzzle_types", "solve_stages", "cases", "user_case_settings"];
    for table in expected {
        let row: (i64,) = sqlx::query_as(
            "SELECT count(*)::bigint FROM information_schema.tables \
             WHERE table_schema = 'public' AND table_name = $1",
        )
        .bind(table)
        .fetch_one(&db.pool)
        .await
        .expect("information_schema query");
        assert_eq!(row.0, 1, "expected table {table} to exist");
    }
}

#[tokio::test]
async fn tier1_tag_check_rejects_unknown_value() {
    let db = TestDb::new().await;
    let stage_id = seed_stage(&db.pool).await;

    let err = sqlx::query(
        r#"
        INSERT INTO cases (solve_stage_id, case_number, algorithm, diagram_data, tier1_tag)
        VALUES ($1, 1, 'R U R''', '{}'::jsonb, 'Q')
        "#,
    )
    .bind(stage_id)
    .execute(&db.pool)
    .await
    .expect_err("insert should fail tier1_tag check");

    let msg = format!("{err}");
    assert!(
        msg.contains("tier1_tag") || msg.contains("check"),
        "expected check-constraint error, got: {msg}"
    );
}

#[tokio::test]
async fn result_rotation_check_rejects_out_of_range() {
    let db = TestDb::new().await;
    let stage_id = seed_stage(&db.pool).await;

    let err = sqlx::query(
        r#"
        INSERT INTO cases
            (solve_stage_id, case_number, algorithm, diagram_data, tier1_tag, result_rotation)
        VALUES ($1, 1, 'R U R''', '{}'::jsonb, '*', 4)
        "#,
    )
    .bind(stage_id)
    .execute(&db.pool)
    .await
    .expect_err("insert should fail result_rotation check");

    let msg = format!("{err}");
    assert!(
        msg.contains("result_rotation") || msg.contains("check"),
        "expected check-constraint error, got: {msg}"
    );
}

#[tokio::test]
async fn display_rotation_check_rejects_out_of_range() {
    let db = TestDb::new().await;
    let stage_id = seed_stage(&db.pool).await;

    let err = sqlx::query(
        r#"
        INSERT INTO cases
            (solve_stage_id, case_number, algorithm, diagram_data, tier1_tag, display_rotation)
        VALUES ($1, 1, 'R U R''', '{}'::jsonb, '*', 4)
        "#,
    )
    .bind(stage_id)
    .execute(&db.pool)
    .await
    .expect_err("insert should fail display_rotation check");

    let msg = format!("{err}");
    assert!(
        msg.contains("display_rotation") || msg.contains("check"),
        "expected check-constraint error, got: {msg}"
    );
}

#[tokio::test]
async fn case_number_unique_within_stage() {
    let db = TestDb::new().await;
    let stage_id = seed_stage(&db.pool).await;

    sqlx::query(
        r#"
        INSERT INTO cases (solve_stage_id, case_number, algorithm, diagram_data, tier1_tag)
        VALUES ($1, 1, 'R', '{}'::jsonb, '*')
        "#,
    )
    .bind(stage_id)
    .execute(&db.pool)
    .await
    .expect("first insert");

    let err = sqlx::query(
        r#"
        INSERT INTO cases (solve_stage_id, case_number, algorithm, diagram_data, tier1_tag)
        VALUES ($1, 1, 'L', '{}'::jsonb, '*')
        "#,
    )
    .bind(stage_id)
    .execute(&db.pool)
    .await
    .expect_err("duplicate case_number should fail");

    let msg = format!("{err}");
    assert!(msg.contains("unique") || msg.contains("23505"), "got: {msg}");
}

#[tokio::test]
async fn user_case_settings_cascades_on_user_delete() {
    let db = TestDb::new().await;
    let stage_id = seed_stage(&db.pool).await;
    let case_id = seed_case(&db.pool, stage_id, 1).await;
    let user_id = seed_user(&db.pool, "alice@example.com").await;

    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, nickname) VALUES ($1, $2, 'Mine')",
    )
    .bind(user_id)
    .bind(case_id)
    .execute(&db.pool)
    .await
    .expect("insert override");

    sqlx::query("DELETE FROM users WHERE id = $1")
        .bind(user_id)
        .execute(&db.pool)
        .await
        .expect("delete user");

    let row = sqlx::query("SELECT count(*)::bigint AS n FROM user_case_settings WHERE user_id = $1")
        .bind(user_id)
        .fetch_one(&db.pool)
        .await
        .expect("count");
    let count: i64 = row.get("n");
    assert_eq!(count, 0, "override row should cascade-delete with user");
}

#[tokio::test]
async fn updated_at_trigger_fires_on_settings_update() {
    let db = TestDb::new().await;
    let stage_id = seed_stage(&db.pool).await;
    let case_id = seed_case(&db.pool, stage_id, 1).await;
    let user_id = seed_user(&db.pool, "bob@example.com").await;

    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, nickname) VALUES ($1, $2, 'First')",
    )
    .bind(user_id)
    .bind(case_id)
    .execute(&db.pool)
    .await
    .expect("insert");

    let before: (chrono::DateTime<chrono::Utc>,) = sqlx::query_as(
        "SELECT updated_at FROM user_case_settings WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user_id)
    .bind(case_id)
    .fetch_one(&db.pool)
    .await
    .expect("select before");

    // Sleep a hair so a new NOW() reading is distinguishable.
    tokio::time::sleep(std::time::Duration::from_millis(20)).await;

    sqlx::query(
        "UPDATE user_case_settings SET nickname = 'Second' WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user_id)
    .bind(case_id)
    .execute(&db.pool)
    .await
    .expect("update");

    let after: (chrono::DateTime<chrono::Utc>,) = sqlx::query_as(
        "SELECT updated_at FROM user_case_settings WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user_id)
    .bind(case_id)
    .fetch_one(&db.pool)
    .await
    .expect("select after");

    assert!(after.0 > before.0, "updated_at should advance on UPDATE");
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

async fn seed_stage(pool: &sqlx::PgPool) -> Uuid {
    // Use a unique puzzle/stage name so this helper is independent of the
    // 3x3/OLL pair the seed migration installs.
    let puzzle: (Uuid,) = sqlx::query_as(
        "INSERT INTO puzzle_types (name) VALUES ('test-puzzle') RETURNING id",
    )
    .fetch_one(pool)
    .await
    .expect("insert puzzle_type");

    let stage: (Uuid,) = sqlx::query_as(
        "INSERT INTO solve_stages (puzzle_type_id, name) VALUES ($1, 'TEST_STAGE') RETURNING id",
    )
    .bind(puzzle.0)
    .fetch_one(pool)
    .await
    .expect("insert solve_stage");

    stage.0
}

async fn seed_case(pool: &sqlx::PgPool, stage_id: Uuid, number: i32) -> Uuid {
    let row: (Uuid,) = sqlx::query_as(
        r#"
        INSERT INTO cases (solve_stage_id, case_number, algorithm, diagram_data, tier1_tag)
        VALUES ($1, $2, 'R U R''', '{"pattern":"XXXXXXXXX"}'::jsonb, '*')
        RETURNING id
        "#,
    )
    .bind(stage_id)
    .bind(number)
    .fetch_one(pool)
    .await
    .expect("insert case");

    row.0
}

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
