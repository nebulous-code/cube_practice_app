//! Case data — the authoritative SQL merge of global `cases` rows with
//! per-user `user_case_settings` overrides. The only place we resolve
//! "global default vs override" — every route returns the merged shape.
//!
//! See docs/milestones/02_case_data_and_browser.md §4 for the JSON shape
//! and §3 for the underlying schema.

use chrono::{DateTime, Utc};
use serde::Serialize;
use serde_json::Value as JsonValue;
use sqlx::PgPool;
use uuid::Uuid;

use crate::error::{AppError, AppResult};

/// API-facing merged case. Identical shape from `GET /cases`,
/// `GET /cases/:id`, and `PATCH /cases/:id/settings`.
#[derive(Debug, Serialize)]
pub struct Case {
    pub id: Uuid,
    pub solve_stage: String,
    pub puzzle_type: String,
    pub case_number: i32,
    pub nickname: Option<String>,
    pub algorithm: String,
    pub result_case_id: Option<Uuid>,
    pub result_case_number: Option<i32>,
    pub result_rotation: i32,
    pub pattern: String,
    pub tier1_tag: String,
    pub tier2_tag: Option<String>,
    pub has_overrides: bool,
}

/// Internal row shape returned by the merge SQL. Stays private so the
/// API `Case` struct is the only thing route code touches.
#[derive(Debug, sqlx::FromRow)]
struct MergedRow {
    id: Uuid,
    solve_stage: String,
    puzzle_type: String,
    case_number: i32,
    nickname: Option<String>,
    algorithm: String,
    result_case_id: Option<Uuid>,
    result_case_number: Option<i32>,
    result_rotation: i32,
    diagram_data: JsonValue,
    tier1_tag: String,
    tier2_tag: Option<String>,
    has_overrides: bool,
    #[allow(dead_code)]
    case_created_at: DateTime<Utc>,
}

impl MergedRow {
    fn into_api(self) -> Case {
        // diagram_data is always `{ "pattern": "<9 chars>" }` for OLL — see
        // §9. Future stages may extend the JSON; for now extract the string.
        let pattern = self
            .diagram_data
            .get("pattern")
            .and_then(JsonValue::as_str)
            .unwrap_or("")
            .to_string();

        Case {
            id: self.id,
            solve_stage: self.solve_stage,
            puzzle_type: self.puzzle_type,
            case_number: self.case_number,
            nickname: self.nickname,
            algorithm: self.algorithm,
            result_case_id: self.result_case_id,
            result_case_number: self.result_case_number,
            result_rotation: self.result_rotation,
            pattern,
            tier1_tag: self.tier1_tag,
            tier2_tag: self.tier2_tag,
            has_overrides: self.has_overrides,
        }
    }
}

/// Override-merge SQL fragment. The merged values use COALESCE so a NULL
/// in `user_case_settings` falls through to the global `cases` value.
/// `has_overrides` is true when an override row exists for this user/case.
/// `result_case_number` is denormalized from the merged result-case for
/// the detail view's "Case 02" label.
const MERGE_SELECT: &str = r#"
SELECT
    c.id                                                  AS id,
    s.name                                                AS solve_stage,
    pt.name                                               AS puzzle_type,
    c.case_number                                         AS case_number,
    COALESCE(ucs.nickname,        c.nickname)             AS nickname,
    COALESCE(ucs.algorithm,       c.algorithm)            AS algorithm,
    COALESCE(ucs.result_case_id,  c.result_case_id)       AS result_case_id,
    rc.case_number                                        AS result_case_number,
    COALESCE(ucs.result_rotation, c.result_rotation)      AS result_rotation,
    c.diagram_data                                        AS diagram_data,
    c.tier1_tag                                           AS tier1_tag,
    COALESCE(ucs.tier2_tag,       c.tier2_tag)            AS tier2_tag,
    (ucs.id IS NOT NULL)                                  AS has_overrides,
    c.created_at                                          AS case_created_at
FROM cases c
JOIN solve_stages s   ON s.id  = c.solve_stage_id
JOIN puzzle_types pt  ON pt.id = s.puzzle_type_id
LEFT JOIN user_case_settings ucs
       ON ucs.case_id = c.id AND ucs.user_id = $1
LEFT JOIN cases rc
       ON rc.id = COALESCE(ucs.result_case_id, c.result_case_id)
"#;

/// Fetch every case for the user, with overrides applied. Sorted by
/// case_number ascending.
pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Case>> {
    let sql = format!("{MERGE_SELECT} ORDER BY c.case_number ASC");
    let rows: Vec<MergedRow> = sqlx::query_as(&sql)
        .bind(user_id)
        .fetch_all(pool)
        .await?;
    Ok(rows.into_iter().map(MergedRow::into_api).collect())
}

/// Fetch one case by id with overrides applied. Returns `NotFound` if
/// no case has that id.
pub async fn get_for_user(
    pool: &PgPool,
    user_id: Uuid,
    case_id: Uuid,
) -> AppResult<Case> {
    let sql = format!("{MERGE_SELECT} WHERE c.id = $2");
    let row: Option<MergedRow> = sqlx::query_as(&sql)
        .bind(user_id)
        .bind(case_id)
        .fetch_optional(pool)
        .await?;
    row.map(MergedRow::into_api).ok_or(AppError::NotFound)
}
