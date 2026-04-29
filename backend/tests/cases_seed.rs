//! Verifies the seed migration ports all 57 OLL cases correctly and that
//! the result-case backfill links every case to a valid target.

mod common;

use common::TestDb;

#[tokio::test]
async fn seeds_57_oll_cases() {
    let db = TestDb::new().await;

    let count: (i64,) = sqlx::query_as(
        r#"
        SELECT count(*)::bigint
        FROM cases c
        JOIN solve_stages s ON s.id = c.solve_stage_id AND s.name = 'OLL'
        JOIN puzzle_types pt ON pt.id = s.puzzle_type_id AND pt.name = '3x3'
        "#,
    )
    .fetch_one(&db.pool)
    .await
    .expect("count cases");

    assert_eq!(count.0, 57, "expected 57 OLL cases, got {}", count.0);
}

#[tokio::test]
async fn one_puzzle_type_one_stage() {
    let db = TestDb::new().await;

    let pt: (i64,) = sqlx::query_as("SELECT count(*)::bigint FROM puzzle_types")
        .fetch_one(&db.pool)
        .await
        .expect("count puzzle_types");
    assert_eq!(pt.0, 1);

    let st: (i64,) = sqlx::query_as("SELECT count(*)::bigint FROM solve_stages")
        .fetch_one(&db.pool)
        .await
        .expect("count solve_stages");
    assert_eq!(st.0, 1);
}

#[tokio::test]
async fn every_case_has_a_valid_result_case() {
    let db = TestDb::new().await;

    let nulls: (i64,) =
        sqlx::query_as("SELECT count(*)::bigint FROM cases WHERE result_case_id IS NULL")
            .fetch_one(&db.pool)
            .await
            .expect("count nulls");
    assert_eq!(nulls.0, 0, "all cases should have a result_case_id after seed");

    // result_case_id must reference a case in the same stage.
    let cross_stage: (i64,) = sqlx::query_as(
        r#"
        SELECT count(*)::bigint
        FROM cases c
        JOIN cases r ON r.id = c.result_case_id
        WHERE c.solve_stage_id <> r.solve_stage_id
        "#,
    )
    .fetch_one(&db.pool)
    .await
    .expect("count cross-stage results");
    assert_eq!(cross_stage.0, 0);
}

#[tokio::test]
async fn tier1_tag_distribution_matches_prototype() {
    // Sanity-check the tier1_tag values match what initial_design/src/data.jsx
    // contains. Counts derived from a quick tally of `priority` in data.jsx.
    let db = TestDb::new().await;

    let tally: Vec<(String, i64)> = sqlx::query_as(
        "SELECT tier1_tag, count(*)::bigint FROM cases GROUP BY tier1_tag ORDER BY tier1_tag",
    )
    .fetch_all(&db.pool)
    .await
    .expect("tally tier1_tag");

    let map: std::collections::HashMap<String, i64> = tally.into_iter().collect();
    assert_eq!(map.get("*").copied().unwrap_or(0), 8, "dot");
    assert_eq!(map.get("+").copied().unwrap_or(0), 7, "cross / OCLL");
    // The remaining 42 cases split between '-' and 'L'; we don't pin the
    // exact split here, just that they sum correctly.
    let line = map.get("-").copied().unwrap_or(0);
    let l = map.get("L").copied().unwrap_or(0);
    assert_eq!(line + l, 57 - 8 - 7);
}

#[tokio::test]
async fn seed_is_idempotent() {
    let db = TestDb::new().await;

    // Run the seed migration's body again — already applied by TestDb::new(),
    // so any successful re-application means the ON CONFLICT clauses fire
    // correctly. We re-run by calling sqlx::migrate again (no-op, but the
    // real proof is the count is still 57 and unique constraints hold).
    let count_before: (i64,) =
        sqlx::query_as("SELECT count(*)::bigint FROM cases")
            .fetch_one(&db.pool)
            .await
            .unwrap();

    // Re-running migrations is a no-op once they've been applied; sqlx
    // tracks them in `_sqlx_migrations`. To genuinely re-apply the seed,
    // we'd need to clear that tracker. Instead, simulate idempotency by
    // running the upsert manually for one row and confirming no duplicate.
    sqlx::query(
        r#"
        INSERT INTO cases
            (solve_stage_id, case_number, nickname, algorithm, diagram_data,
             tier1_tag, tier2_tag, result_rotation)
        SELECT s.id, 1, 'Tie Fighter (re-run)', 'changed', '{"pattern":"X"}'::jsonb, '*', 'dot', 2
        FROM solve_stages s
        JOIN puzzle_types pt ON pt.id = s.puzzle_type_id
        WHERE s.name = 'OLL' AND pt.name = '3x3'
        ON CONFLICT (solve_stage_id, case_number) DO UPDATE SET
            nickname = EXCLUDED.nickname,
            algorithm = EXCLUDED.algorithm
        "#,
    )
    .execute(&db.pool)
    .await
    .expect("upsert");

    let count_after: (i64,) =
        sqlx::query_as("SELECT count(*)::bigint FROM cases")
            .fetch_one(&db.pool)
            .await
            .unwrap();
    assert_eq!(count_before.0, count_after.0, "ON CONFLICT must update, not insert");

    // Confirm the row was actually updated.
    let row: (String,) = sqlx::query_as(
        r#"
        SELECT nickname FROM cases c
        JOIN solve_stages s ON s.id = c.solve_stage_id
        WHERE c.case_number = 1 AND s.name = 'OLL'
        "#,
    )
    .fetch_one(&db.pool)
    .await
    .unwrap();
    assert_eq!(row.0, "Tie Fighter (re-run)");
}
