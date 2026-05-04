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
    /// Multi-valued tags, normalized (lowercase + trim + dedupe).
    /// Per-user override fully replaces the global set; an empty user
    /// override is coerced to NULL by the resolver (no "explicit empty
    /// override" state). See milestone 04 §3.
    pub tags: Vec<String>,
    pub has_overrides: bool,
    /// Per-user SM-2 state — see docs/milestones/03_core_study_loop.md §3.
    pub state: CaseState,
}

/// Maximum length of a single tag, in characters. Enforced by
/// `normalize_tags` — over-cap inputs return Err.
pub const TAG_MAX_LEN: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum CaseState {
    NotStarted,
    Due,
    Learning,
    Mastered,
}

impl CaseState {
    fn from_sql(s: &str) -> Self {
        match s {
            "not_started" => CaseState::NotStarted,
            "due" => CaseState::Due,
            "learning" => CaseState::Learning,
            "mastered" => CaseState::Mastered,
            other => panic!("unexpected state from SQL: {other}"),
        }
    }
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
    tags: Vec<String>,
    has_overrides: bool,
    state: String,
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
            tags: self.tags,
            has_overrides: self.has_overrides,
            state: CaseState::from_sql(&self.state),
        }
    }
}

/// Override-merge SQL fragment. The merged values use COALESCE so a NULL
/// in `user_case_settings` falls through to the global `cases` value.
/// `has_overrides` is true when an override row exists for this user/case.
/// `result_case_number` is denormalized from the merged result-case for
/// the detail view's "Case 02" label. `state` derives the SM-2 state from
/// the LEFT JOIN to `user_case_progress` (21-day threshold for mastered
/// per outstanding_decision.md §1.3).
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
    COALESCE(ucs.tags,            c.tags)                 AS tags,
    (ucs.id IS NOT NULL)                                  AS has_overrides,
    CASE
        WHEN ucp.id IS NULL                  THEN 'not_started'
        WHEN ucp.due_date <= CURRENT_DATE    THEN 'due'
        WHEN ucp.interval_days < 21          THEN 'learning'
        ELSE                                      'mastered'
    END                                                   AS state,
    c.created_at                                          AS case_created_at
FROM cases c
JOIN solve_stages s   ON s.id  = c.solve_stage_id
JOIN puzzle_types pt  ON pt.id = s.puzzle_type_id
LEFT JOIN user_case_settings ucs
       ON ucs.case_id = c.id AND ucs.user_id = $1
LEFT JOIN user_case_progress ucp
       ON ucp.case_id = c.id AND ucp.user_id = $1
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

/// Fetch only cases currently in the `due` state for the user. Sorted by
/// `due_date` ascending so the oldest-due cards come first in the queue.
pub async fn list_due_for_user(pool: &PgPool, user_id: Uuid) -> AppResult<Vec<Case>> {
    let sql = format!(
        "{MERGE_SELECT} WHERE ucp.id IS NOT NULL AND ucp.due_date <= CURRENT_DATE \
         ORDER BY ucp.due_date ASC, c.case_number ASC"
    );
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

/// Per-field patch. `None` = "leave whatever was there alone";
/// `Some(None)` = "clear the override"; `Some(Some(v))` = "set override".
/// Mirrors the `Option<Option<T>>` shape used by the PATCH route.
///
/// `tags`: callers should pass already-normalized vectors. The route
/// layer is responsible for running `normalize_tags` and coercing an
/// empty result to `Some(None)` (= clear the override).
#[derive(Debug, Default, Clone)]
pub struct SettingsPatch {
    pub nickname: Option<Option<String>>,
    pub algorithm: Option<Option<String>>,
    pub result_case_id: Option<Option<Uuid>>,
    pub result_rotation: Option<Option<i32>>,
    pub tags: Option<Option<Vec<String>>>,
}

/// Resolved override row — every field carries its post-merge value.
#[derive(Debug, Default)]
struct ResolvedSettings {
    nickname: Option<String>,
    algorithm: Option<String>,
    result_case_id: Option<Uuid>,
    result_rotation: Option<i32>,
    tags: Option<Vec<String>>,
}

impl ResolvedSettings {
    fn is_all_null(&self) -> bool {
        self.nickname.is_none()
            && self.algorithm.is_none()
            && self.result_case_id.is_none()
            && self.result_rotation.is_none()
            && self.tags.is_none()
    }
}

/// Normalize a tag input vector: trim each entry, lowercase ASCII
/// letters (Unicode passes through), drop empty entries, dedupe
/// preserving first-seen order. Returns `Err` with a user-facing message
/// when any tag exceeds `TAG_MAX_LEN` characters.
pub fn normalize_tags(input: Vec<String>) -> Result<Vec<String>, String> {
    let mut out: Vec<String> = Vec::with_capacity(input.len());
    for raw in input {
        let mut trimmed = raw.trim().to_string();
        trimmed.make_ascii_lowercase();
        if trimmed.is_empty() {
            continue;
        }
        if trimmed.chars().count() > TAG_MAX_LEN {
            return Err(format!("Each tag must be {TAG_MAX_LEN} characters or fewer."));
        }
        if !out.contains(&trimmed) {
            out.push(trimmed);
        }
    }
    Ok(out)
}

/// Apply a `SettingsPatch` for the (user, case) pair. Reads the existing
/// override row (if any), merges the patch with the documented `Option<Option>`
/// semantics, then either deletes the row (all fields null) or upserts it.
/// Returns the freshly-merged `Case`.
///
/// Validates that:
///   - The case exists.
///   - `result_case_id` (if set) refers to a case in the same `solve_stage`.
pub async fn update_settings(
    pool: &PgPool,
    user_id: Uuid,
    case_id: Uuid,
    patch: SettingsPatch,
) -> AppResult<Case> {
    let stage_id: Option<(Uuid,)> =
        sqlx::query_as("SELECT solve_stage_id FROM cases WHERE id = $1")
            .bind(case_id)
            .fetch_optional(pool)
            .await?;
    let (stage_id,) = stage_id.ok_or(AppError::NotFound)?;

    if let Some(Some(rcid)) = patch.result_case_id {
        let target: Option<(Uuid,)> =
            sqlx::query_as("SELECT solve_stage_id FROM cases WHERE id = $1")
                .bind(rcid)
                .fetch_optional(pool)
                .await?;
        let invalid = match target {
            None => true,
            Some((target_stage,)) => target_stage != stage_id,
        };
        if invalid {
            let mut fields = std::collections::HashMap::new();
            fields.insert(
                "result_case_id".into(),
                "Must reference a case in the same solve stage.".into(),
            );
            return Err(AppError::Validation(fields));
        }
    }

    // Read current override row (may not exist).
    let existing: Option<(Option<String>, Option<String>, Option<Uuid>, Option<i32>, Option<Vec<String>>)> =
        sqlx::query_as(
            "SELECT nickname, algorithm, result_case_id, result_rotation, tags \
             FROM user_case_settings WHERE user_id = $1 AND case_id = $2",
        )
        .bind(user_id)
        .bind(case_id)
        .fetch_optional(pool)
        .await?;

    let resolved = resolve(&patch, existing);

    if resolved.is_all_null() {
        sqlx::query("DELETE FROM user_case_settings WHERE user_id = $1 AND case_id = $2")
            .bind(user_id)
            .bind(case_id)
            .execute(pool)
            .await?;
    } else {
        sqlx::query(
            r#"
            INSERT INTO user_case_settings
                (user_id, case_id, nickname, algorithm, result_case_id, result_rotation, tags)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            ON CONFLICT (user_id, case_id) DO UPDATE SET
                nickname        = EXCLUDED.nickname,
                algorithm       = EXCLUDED.algorithm,
                result_case_id  = EXCLUDED.result_case_id,
                result_rotation = EXCLUDED.result_rotation,
                tags            = EXCLUDED.tags
            "#,
        )
        .bind(user_id)
        .bind(case_id)
        .bind(&resolved.nickname)
        .bind(&resolved.algorithm)
        .bind(resolved.result_case_id)
        .bind(resolved.result_rotation)
        .bind(resolved.tags.as_deref())
        .execute(pool)
        .await?;
    }

    get_for_user(pool, user_id, case_id).await
}

fn resolve(
    patch: &SettingsPatch,
    existing: Option<(Option<String>, Option<String>, Option<Uuid>, Option<i32>, Option<Vec<String>>)>,
) -> ResolvedSettings {
    let (e_nick, e_alg, e_rcid, e_rot, e_tags) = existing.unwrap_or_default();
    ResolvedSettings {
        nickname: apply_string(&patch.nickname, e_nick),
        algorithm: apply_string(&patch.algorithm, e_alg),
        result_case_id: apply_copy(patch.result_case_id, e_rcid),
        result_rotation: apply_copy(patch.result_rotation, e_rot),
        tags: apply_clone(patch.tags.as_ref(), e_tags),
    }
}

fn apply_clone<T: Clone>(patch: Option<&Option<T>>, existing: Option<T>) -> Option<T> {
    match patch {
        None => existing,
        Some(None) => None,
        Some(Some(v)) => Some(v.clone()),
    }
}

fn apply_string(patch: &Option<Option<String>>, existing: Option<String>) -> Option<String> {
    match patch {
        None => existing,
        Some(None) => None,
        Some(Some(v)) => Some(v.clone()),
    }
}

fn apply_copy<T: Copy>(patch: Option<Option<T>>, existing: Option<T>) -> Option<T> {
    match patch {
        None => existing,
        Some(None) => None,
        Some(Some(v)) => Some(v),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalize_tags_empty_input() {
        assert_eq!(normalize_tags(vec![]).unwrap(), Vec::<String>::new());
    }

    #[test]
    fn normalize_tags_lowercases_ascii() {
        assert_eq!(
            normalize_tags(vec!["Fish".into(), "FISH".into()]).unwrap(),
            vec!["fish"],
        );
    }

    #[test]
    fn normalize_tags_trims_whitespace() {
        assert_eq!(
            normalize_tags(vec!["  fish  ".into(), "\tneeds work\n".into()]).unwrap(),
            vec!["fish", "needs work"],
        );
    }

    #[test]
    fn normalize_tags_drops_empty_after_trim() {
        assert_eq!(
            normalize_tags(vec!["".into(), "   ".into(), "fish".into()]).unwrap(),
            vec!["fish"],
        );
    }

    #[test]
    fn normalize_tags_dedupes_preserving_first_seen_order() {
        assert_eq!(
            normalize_tags(vec![
                "fish".into(),
                "needs work".into(),
                "fish".into(),
                "Fish".into(), // also a duplicate after lowercase
            ])
            .unwrap(),
            vec!["fish", "needs work"],
        );
    }

    #[test]
    fn normalize_tags_passes_unicode_through() {
        // Non-ASCII letters aren't lowercased — "Δ" stays "Δ".
        assert_eq!(
            normalize_tags(vec!["Δ".into(), "café".into()]).unwrap(),
            vec!["Δ", "café"],
        );
    }

    #[test]
    fn normalize_tags_rejects_over_cap() {
        let too_long: String = "a".repeat(TAG_MAX_LEN + 1);
        let err = normalize_tags(vec![too_long]).expect_err("should reject");
        assert!(err.contains(&TAG_MAX_LEN.to_string()), "got: {err}");
    }

    #[test]
    fn normalize_tags_accepts_at_cap() {
        let at_cap: String = "a".repeat(TAG_MAX_LEN);
        assert_eq!(normalize_tags(vec![at_cap.clone()]).unwrap(), vec![at_cap]);
    }
}
