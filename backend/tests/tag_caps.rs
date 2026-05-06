//! M6 B2 — per-user tag caps. See docs/milestones/06_guest_mode.md §2 / §4.
//!
//! Three caps:
//!   - per-tag length: TAG_MAX_LEN = 50 chars (already enforced via
//!     normalize_tags; covered in cases/mod.rs unit tests)
//!   - per-user distinct tags: MAX_DISTINCT_TAGS_PER_USER = 100
//!   - per-user total tag-links: MAX_TAG_LINKS_PER_USER = 1000

mod common;

use common::TestDb;
use cube_backend::cases::{
    self, SettingsPatch, MAX_DISTINCT_TAGS_PER_USER, MAX_TAG_LINKS_PER_USER, TAG_MAX_LEN,
};
use cube_backend::error::AppError;
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

async fn case_id(pool: &sqlx::PgPool, n: i32) -> Uuid {
    let row: (Uuid,) = sqlx::query_as("SELECT id FROM cases WHERE case_number = $1")
        .bind(n)
        .fetch_one(pool)
        .await
        .expect("case lookup");
    row.0
}

/// Direct-insert tags onto a user_case_settings row for setup purposes —
/// bypasses validate_tag_caps so we can construct over/under-cap states
/// for the test.
async fn force_tags(
    pool: &sqlx::PgPool,
    user_id: Uuid,
    c_id: Uuid,
    tags: Vec<String>,
) {
    sqlx::query(
        "INSERT INTO user_case_settings (user_id, case_id, tags) VALUES ($1, $2, $3) \
         ON CONFLICT (user_id, case_id) DO UPDATE SET tags = EXCLUDED.tags",
    )
    .bind(user_id)
    .bind(c_id)
    .bind(&tags)
    .execute(pool)
    .await
    .expect("force tags");
}

fn patch_tags(tags: Vec<String>) -> SettingsPatch {
    SettingsPatch {
        nickname: None,
        algorithm: None,
        result_case_id: None,
        result_rotation: None,
        tags: Some(Some(tags)),
    }
}

// ─── single-write boundaries ─────────────────────────────────────────────────

#[tokio::test]
async fn at_distinct_cap_is_accepted() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let c = case_id(&db.pool, 1).await;

    // Exactly 100 distinct tags on a single case.
    let tags: Vec<String> = (0..MAX_DISTINCT_TAGS_PER_USER)
        .map(|i| format!("tag{i}"))
        .collect();

    cases::update_settings(&db.pool, user, c, patch_tags(tags))
        .await
        .expect("at-cap distinct should pass");
}

#[tokio::test]
async fn one_over_distinct_cap_rejects() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let c = case_id(&db.pool, 1).await;

    let tags: Vec<String> = (0..=MAX_DISTINCT_TAGS_PER_USER)
        .map(|i| format!("tag{i}"))
        .collect();

    let err = cases::update_settings(&db.pool, user, c, patch_tags(tags))
        .await
        .expect_err("one over should reject");
    match err {
        AppError::Validation(fields) => {
            let msg = fields.get("tags").expect("tags field");
            assert!(msg.contains("Distinct"), "got: {msg}");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

#[tokio::test]
async fn at_link_cap_across_cases_is_accepted() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    // Pre-load 999 tag-links across 50 other cases via direct insert,
    // each with 20-tag arrays drawn from a shared pool of 80 names. That
    // gives 999 links + ~80 distinct.
    let pool_names: Vec<String> = (0..80).map(|i| format!("p{i}")).collect();
    let mut links_used = 0usize;
    for n in 2..=50 {
        let take = 20.min(MAX_TAG_LINKS_PER_USER - links_used - 1);
        if take == 0 {
            break;
        }
        let tags: Vec<String> = pool_names.iter().take(take).cloned().collect();
        force_tags(&db.pool, user, case_id(&db.pool, n).await, tags).await;
        links_used += take;
    }
    // Top up to exactly 999 on case 51 if we didn't quite hit it.
    let remaining = MAX_TAG_LINKS_PER_USER - 1 - links_used;
    if remaining > 0 {
        let tags: Vec<String> = pool_names.iter().take(remaining).cloned().collect();
        force_tags(&db.pool, user, case_id(&db.pool, 51).await, tags).await;
    }

    // Now apply a 1-tag patch via update_settings on case 1: total = 1000
    // exactly — at the cap, must be accepted.
    cases::update_settings(
        &db.pool,
        user,
        case_id(&db.pool, 1).await,
        patch_tags(vec!["p0".into()]),
    )
    .await
    .expect("at-cap link total should pass");
}

#[tokio::test]
async fn one_over_link_cap_rejects() {
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;

    // Same setup as the at-cap test, but push to 1000 pre-existing links.
    let pool_names: Vec<String> = (0..80).map(|i| format!("p{i}")).collect();
    let mut links_used = 0usize;
    for n in 2..=50 {
        let take = 20.min(MAX_TAG_LINKS_PER_USER - links_used);
        if take == 0 {
            break;
        }
        let tags: Vec<String> = pool_names.iter().take(take).cloned().collect();
        force_tags(&db.pool, user, case_id(&db.pool, n).await, tags).await;
        links_used += take;
    }
    let remaining = MAX_TAG_LINKS_PER_USER - links_used;
    if remaining > 0 {
        let tags: Vec<String> = pool_names.iter().take(remaining).cloned().collect();
        force_tags(&db.pool, user, case_id(&db.pool, 51).await, tags).await;
    }

    // 1 more tag via update_settings on case 1: total = 1001, must reject.
    let err = cases::update_settings(
        &db.pool,
        user,
        case_id(&db.pool, 1).await,
        patch_tags(vec!["p0".into()]),
    )
    .await
    .expect_err("one over should reject");
    match err {
        AppError::Validation(fields) => {
            let msg = fields.get("tags").expect("tags field");
            assert!(msg.contains("tag-links"), "got: {msg}");
        }
        other => panic!("expected Validation, got {other:?}"),
    }
}

// ─── per-tag length cap ──────────────────────────────────────────────────────

#[tokio::test]
async fn tag_at_length_cap_accepted() {
    let at: String = "a".repeat(TAG_MAX_LEN);
    let normalized = cases::normalize_tags(vec![at.clone()]).expect("at-cap accepted");
    assert_eq!(normalized, vec![at]);
}

#[tokio::test]
async fn tag_over_length_cap_rejected_at_normalize() {
    let over: String = "a".repeat(TAG_MAX_LEN + 1);
    cases::normalize_tags(vec![over]).expect_err("over-cap should reject");
}

// ─── isolation: another user's tags don't count ─────────────────────────────

#[tokio::test]
async fn other_users_tags_dont_count_against_my_cap() {
    let db = TestDb::new().await;
    let alice = seed_user(&db.pool, "alice@example.com").await;
    let bob = seed_user(&db.pool, "bob@example.com").await;

    // Bob crams 100 distinct tags onto a single case.
    let bob_tags: Vec<String> = (0..MAX_DISTINCT_TAGS_PER_USER)
        .map(|i| format!("b{i}"))
        .collect();
    cases::update_settings(
        &db.pool,
        bob,
        case_id(&db.pool, 1).await,
        patch_tags(bob_tags),
    )
    .await
    .expect("bob fills his cap");

    // Alice should still be able to add tags freely — her cap is per-user.
    cases::update_settings(
        &db.pool,
        alice,
        case_id(&db.pool, 1).await,
        patch_tags(vec!["fish".into(), "needs work".into()]),
    )
    .await
    .expect("alice's writes shouldn't see bob's tags");
}

// ─── self-replacement: editing a row's existing tags doesn't double-count ───

#[tokio::test]
async fn replacing_existing_tags_does_not_double_count() {
    // Edge case in validate_tag_caps: when a user updates a case that
    // already has tags, the existing tags on THAT case must not be
    // tallied alongside the new tags. The query excludes the row being
    // updated; this test guards that.
    let db = TestDb::new().await;
    let user = seed_user(&db.pool, "alice@example.com").await;
    let c = case_id(&db.pool, 1).await;

    // Start at the cap with 100 distinct tags on case 1.
    let initial: Vec<String> =
        (0..MAX_DISTINCT_TAGS_PER_USER).map(|i| format!("t{i}")).collect();
    cases::update_settings(&db.pool, user, c, patch_tags(initial))
        .await
        .expect("at-cap initial write");

    // Replace with a different 100-tag set on the same case. If the
    // validator double-counted, this would read as 200 distinct and
    // reject. It must accept.
    let replacement: Vec<String> = (100..200).map(|i| format!("t{i}")).collect();
    cases::update_settings(&db.pool, user, c, patch_tags(replacement))
        .await
        .expect("self-replacement at the cap should pass");
}
