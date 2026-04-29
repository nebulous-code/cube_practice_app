//! Study endpoints — due-card retrieval and review submission. Wraps the
//! pure SM-2 logic in `srs` with the DB I/O and the streak update.
//!
//! See docs/milestones/03_core_study_loop.md §4–§5 for the spec.

use chrono::{DateTime, NaiveDate, Utc};
use serde::Serialize;
use sqlx::{PgPool, Postgres, Transaction};
use uuid::Uuid;

use crate::cases::{self, Case};
use crate::error::{AppError, AppResult};
use crate::srs::{self, Grade, ProgressState};

/// Streak summary returned alongside study responses. Read off `users` —
/// `streak_count` is the consecutive-day count; `last_practice_date` is the
/// date of the most recent review (NULL for users who've never reviewed).
#[derive(Debug, Serialize)]
pub struct Streak {
    pub count: i32,
    pub last_practice_date: Option<NaiveDate>,
}

/// Response shape for `GET /study/due`.
#[derive(Debug, Serialize)]
pub struct DueResponse {
    pub cases: Vec<Case>,
    pub streak: Streak,
}

/// Response shape for `POST /study/:case_id/review`.
#[derive(Debug, Serialize)]
pub struct ReviewResponse {
    pub case: Case,
    pub streak: Streak,
}

/// Cases due today for `user_id`, oldest-due first. Joined to the merge
/// view so each entry carries the full `Case` shape.
pub async fn due_for_user(pool: &PgPool, user_id: Uuid) -> AppResult<DueResponse> {
    let cases = cases::list_due_for_user(pool, user_id).await?;
    let streak = read_streak(pool, user_id).await?;
    Ok(DueResponse { cases, streak })
}

/// Apply a review for `(user_id, case_id)`. If no progress row exists, one
/// is created with default state and immediately graded. SM-2 update + streak
/// tick happen atomically. Returns the post-review merged case + streak.
pub async fn apply_review(
    pool: &PgPool,
    user_id: Uuid,
    case_id: Uuid,
    grade: Grade,
    today: NaiveDate,
) -> AppResult<ReviewResponse> {
    // Verify the case exists before touching anything else.
    let case_exists: Option<(Uuid,)> =
        sqlx::query_as("SELECT id FROM cases WHERE id = $1")
            .bind(case_id)
            .fetch_optional(pool)
            .await?;
    if case_exists.is_none() {
        return Err(AppError::NotFound);
    }

    let mut tx = pool.begin().await?;

    let prev = read_or_default_progress(&mut tx, user_id, case_id, today).await?;
    let next = srs::next_state(prev, grade, today);
    let now = Utc::now();
    upsert_progress(&mut tx, user_id, case_id, &next, grade, now).await?;
    update_streak(&mut tx, user_id, today).await?;

    tx.commit().await?;

    let case = cases::get_for_user(pool, user_id, case_id).await?;
    let streak = read_streak(pool, user_id).await?;
    Ok(ReviewResponse { case, streak })
}

async fn read_or_default_progress(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    case_id: Uuid,
    today: NaiveDate,
) -> AppResult<ProgressState> {
    let row: Option<(f64, i32, i32, NaiveDate)> = sqlx::query_as(
        "SELECT ease_factor, interval_days, repetitions, due_date \
         FROM user_case_progress WHERE user_id = $1 AND case_id = $2",
    )
    .bind(user_id)
    .bind(case_id)
    .fetch_optional(&mut **tx)
    .await?;

    Ok(match row {
        Some((ease, interval, reps, due)) => ProgressState {
            ease_factor: ease,
            interval_days: interval,
            repetitions: reps,
            due_date: due,
        },
        None => ProgressState::initial(today),
    })
}

async fn upsert_progress(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    case_id: Uuid,
    next: &ProgressState,
    grade: Grade,
    now: DateTime<Utc>,
) -> AppResult<()> {
    sqlx::query(
        r#"
        INSERT INTO user_case_progress
            (user_id, case_id, ease_factor, interval_days, repetitions,
             due_date, last_grade, last_reviewed)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
        ON CONFLICT (user_id, case_id) DO UPDATE SET
            ease_factor   = EXCLUDED.ease_factor,
            interval_days = EXCLUDED.interval_days,
            repetitions   = EXCLUDED.repetitions,
            due_date      = EXCLUDED.due_date,
            last_grade    = EXCLUDED.last_grade,
            last_reviewed = EXCLUDED.last_reviewed
        "#,
    )
    .bind(user_id)
    .bind(case_id)
    .bind(next.ease_factor)
    .bind(next.interval_days)
    .bind(next.repetitions)
    .bind(next.due_date)
    .bind(grade.as_u8() as i32)
    .bind(now)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

/// Streak rules from docs/Cube_Practice_Design_Doc.md §4 "Behavioral notes":
///   - last_practice_date is None → streak = 1
///   - last_practice_date == today → unchanged
///   - last_practice_date == today - 1 → streak += 1
///   - otherwise → streak = 1
/// Always sets last_practice_date = today afterward.
async fn update_streak(
    tx: &mut Transaction<'_, Postgres>,
    user_id: Uuid,
    today: NaiveDate,
) -> AppResult<()> {
    let row: Option<(i32, Option<NaiveDate>)> = sqlx::query_as(
        "SELECT streak_count, last_practice_date FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(&mut **tx)
    .await?;
    let (current, last) = row.ok_or(AppError::Unauthorized)?;

    let new_count = match last {
        None => 1,
        Some(prev) if prev == today => current,
        Some(prev) if prev.succ_opt() == Some(today) => current + 1,
        _ => 1,
    };

    sqlx::query(
        "UPDATE users SET streak_count = $2, last_practice_date = $3 WHERE id = $1",
    )
    .bind(user_id)
    .bind(new_count)
    .bind(today)
    .execute(&mut **tx)
    .await?;
    Ok(())
}

async fn read_streak(pool: &PgPool, user_id: Uuid) -> AppResult<Streak> {
    let row: Option<(i32, Option<NaiveDate>)> = sqlx::query_as(
        "SELECT streak_count, last_practice_date FROM users WHERE id = $1",
    )
    .bind(user_id)
    .fetch_optional(pool)
    .await?;
    let (count, last) = row.ok_or(AppError::Unauthorized)?;
    Ok(Streak {
        count,
        last_practice_date: last,
    })
}
