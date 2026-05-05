//! Behavioral tests for the M5 onboarding-complete endpoint helper.
//! See docs/milestones/05_polish_and_static_pages.md §7.
//!
//! Tests target `cube_backend::onboarding::mark_seen`, the library function
//! the route handler delegates to. The HTTP auth gate is shared with every
//! other authed endpoint and not retested here.

mod common;

use common::TestDb;
use cube_backend::onboarding;
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

async fn has_seen(pool: &PgPool, user_id: Uuid) -> bool {
    let row: (bool,) =
        sqlx::query_as("SELECT has_seen_onboarding FROM users WHERE id = $1")
            .bind(user_id)
            .fetch_one(pool)
            .await
            .expect("query has_seen_onboarding");
    row.0
}

#[tokio::test]
async fn mark_seen_flips_flag_to_true() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    assert!(!has_seen(&db.pool, user).await, "starts false");

    onboarding::mark_seen(&db.pool, user).await.unwrap();

    assert!(has_seen(&db.pool, user).await, "flipped to true");
}

#[tokio::test]
async fn mark_seen_is_idempotent() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    onboarding::mark_seen(&db.pool, user).await.unwrap();
    onboarding::mark_seen(&db.pool, user).await.unwrap();

    assert!(has_seen(&db.pool, user).await);
}

#[tokio::test]
async fn mark_seen_isolates_users() {
    let db = TestDb::new().await;
    let alice = seed_user(&db.pool, "alice@example.com").await;
    let bob = seed_user(&db.pool, "bob@example.com").await;

    onboarding::mark_seen(&db.pool, alice).await.unwrap();

    assert!(has_seen(&db.pool, alice).await);
    assert!(
        !has_seen(&db.pool, bob).await,
        "bob's flag must not flip when alice completes onboarding"
    );
}

#[tokio::test]
async fn mark_seen_for_nonexistent_user_is_a_noop() {
    // The endpoint contract is "no errors beyond the standard auth gate."
    // The gate guarantees the user exists, but defensively the helper should
    // not blow up on a missing row — UPDATE … WHERE id = $1 with no match is
    // simply zero rows updated.
    let db = TestDb::new().await;
    let phantom = Uuid::new_v4();

    onboarding::mark_seen(&db.pool, phantom)
        .await
        .expect("noop UPDATE should succeed");
}
