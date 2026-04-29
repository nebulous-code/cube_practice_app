//! Smoke test for the integration-test harness itself: make sure a fresh
//! database can be provisioned, migrated, and queried.

mod common;

use common::TestDb;

#[tokio::test]
async fn harness_provisions_and_migrates() {
    let db = TestDb::new().await;

    let one: (i32,) = sqlx::query_as("SELECT 1::int4")
        .fetch_one(&db.pool)
        .await
        .expect("select 1");
    assert_eq!(one.0, 1);

    let users: (i64,) = sqlx::query_as("SELECT count(*)::bigint FROM users")
        .fetch_one(&db.pool)
        .await
        .expect("count users");
    assert_eq!(users.0, 0);

    let sessions: (i64,) = sqlx::query_as("SELECT count(*)::bigint FROM sessions")
        .fetch_one(&db.pool)
        .await
        .expect("count sessions");
    assert_eq!(sessions.0, 0);
}
