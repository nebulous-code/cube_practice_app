//! Progress endpoints — per-user state aggregation. The state derivation
//! itself lives in `cases::MERGE_SELECT` (one CASE expression). This
//! module just rolls those rows up into counts and surfaces them
//! alongside the user's streak.
//!
//! See docs/milestones/04_dashboard_progress_free_study_tags.md §4.

use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::cases::{self, Case, CaseState};
use crate::error::AppResult;
use crate::study::{self, Streak};

/// Per-state count breakdown for a user. `total` is the count of canonical
/// `cases` rows (currently 57 for OLL); the four state counts always sum
/// to `total`.
#[derive(Debug, Serialize, Default)]
pub struct ProgressCounts {
    pub not_started: i64,
    pub learning: i64,
    pub due: i64,
    pub mastered: i64,
}

/// Response shape for `GET /progress`.
#[derive(Debug, Serialize)]
pub struct ProgressSummary {
    pub summary: ProgressCounts,
    pub total: i64,
    pub streak: Streak,
}

/// Aggregate the per-user case-state counts and bundle the streak. Single
/// `GROUP BY` over the same merge that backs `/cases` so the state
/// derivation stays in one place.
pub async fn summary_for_user(pool: &PgPool, user_id: Uuid) -> AppResult<ProgressSummary> {
    let rows: Vec<(String, i64)> = sqlx::query_as(
        r#"
        SELECT
            CASE
                WHEN ucp.id IS NULL                  THEN 'not_started'
                WHEN ucp.due_date <= CURRENT_DATE    THEN 'due'
                WHEN ucp.interval_days < 21          THEN 'learning'
                ELSE                                      'mastered'
            END AS state,
            count(*)::bigint AS n
        FROM cases c
        LEFT JOIN user_case_progress ucp
               ON ucp.case_id = c.id AND ucp.user_id = $1
        GROUP BY state
        "#,
    )
    .bind(user_id)
    .fetch_all(pool)
    .await?;

    let mut counts = ProgressCounts::default();
    let mut total: i64 = 0;
    for (state, n) in rows {
        total += n;
        match state.as_str() {
            "not_started" => counts.not_started = n,
            "due" => counts.due = n,
            "learning" => counts.learning = n,
            "mastered" => counts.mastered = n,
            other => panic!("unexpected state from SQL: {other}"),
        }
    }

    let streak = study::read_streak_public(pool, user_id).await?;

    Ok(ProgressSummary {
        summary: counts,
        total,
        streak,
    })
}

/// Filtered list for `GET /progress/cases?state=…`. When `state` is
/// `None`, returns every case (same as `/cases`). Otherwise filters the
/// merge to only rows in the named state.
pub async fn cases_for_user(
    pool: &PgPool,
    user_id: Uuid,
    state: Option<CaseState>,
) -> AppResult<Vec<Case>> {
    match state {
        None => cases::list_for_user(pool, user_id).await,
        Some(s) => cases::list_for_user_in_state(pool, user_id, s).await,
    }
}
