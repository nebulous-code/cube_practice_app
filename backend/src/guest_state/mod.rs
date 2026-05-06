//! Guest-mode state — the JSON blob a frontend in guest mode persists in
//! localStorage and ships up at register or merge time. See
//! `docs/milestones/06_guest_mode.md` §4 for the API surface and §5 for
//! the on-device shape.
//!
//! Two operations live here:
//!   - `validate(&self)` — pure, stateless; runs every bound and shape check.
//!   - `import(&self, tx, user_id)` — runs inside a transaction; translates
//!     case_number → case_id via the seed table, normalizes tags, writes
//!     `user_case_settings` + `user_case_progress` + the `users` streak/
//!     onboarding columns.
//!
//! Tag-cap enforcement uses the same constants as `PATCH /cases/:id/settings`
//! (see `cases::MAX_DISTINCT_TAGS_PER_USER` etc.) so authed and guest paths
//! yield consistent data.

use std::collections::{HashMap, HashSet};

use chrono::{DateTime, NaiveDate, Utc};
use serde::Deserialize;
use sqlx::{Postgres, Transaction};
use uuid::Uuid;

use crate::cases::{
    normalize_tags, MAX_DISTINCT_TAGS_PER_USER, MAX_TAG_LINKS_PER_USER, TAG_MAX_LEN,
};
use crate::error::{AppError, AppResult};

/// Current schema version. Backend rejects any other value (per Q6 — frontend
/// is responsible for forward-migrating before sending).
pub const SCHEMA_VERSION: u32 = 1;

/// Defensive upper bound on the JSON blob size. Real blobs run < 50 KB even
/// with all 57 cases reviewed; 256 KB leaves headroom for future fields
/// without exposing a denial-of-service vector via gigantic POST bodies.
pub const MAX_BLOB_BYTES: usize = 256 * 1024;

/// Lower SM-2 ease bound. Mirrors the Rust SRS module.
const EASE_FACTOR_MIN: f64 = 1.3;
/// Upper SM-2 ease bound. Anything above this is rejected as malformed.
const EASE_FACTOR_MAX: f64 = 5.0;

/// Top-level guest state. Settings + progress dicts are keyed by `case_number`
/// (1–57) as a string — the same shape the frontend writes into localStorage.
#[derive(Debug, Deserialize)]
pub struct GuestState {
    pub version: u32,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub created_at: Option<DateTime<Utc>>,
    #[serde(default)]
    pub streak_count: i32,
    #[serde(default)]
    pub last_practice_date: Option<NaiveDate>,
    #[serde(default)]
    pub onboarding_completed: bool,
    #[serde(default)]
    pub settings: HashMap<String, GuestSettings>,
    #[serde(default)]
    pub progress: HashMap<String, GuestProgress>,
}

#[derive(Debug, Deserialize)]
pub struct GuestSettings {
    #[serde(default)]
    pub nickname: Option<String>,
    #[serde(default)]
    pub algorithm: Option<String>,
    #[serde(default)]
    pub result_case_number: Option<i32>,
    #[serde(default)]
    pub result_rotation: Option<i32>,
    #[serde(default)]
    pub tags: Vec<String>,
}

#[derive(Debug, Deserialize)]
pub struct GuestProgress {
    pub ease_factor: f64,
    pub interval_days: i32,
    pub repetitions: i32,
    pub due_date: NaiveDate,
    #[serde(default)]
    pub last_grade: Option<i32>,
    #[serde(default)]
    pub last_reviewed: Option<DateTime<Utc>>,
}

impl GuestState {
    /// Pure validation pass — no DB hits, runs synchronously. Surfaces a
    /// single `AppError::Validation` with one or more keyed messages on
    /// failure. Bounds enforced match `docs/milestones/06_guest_mode.md` §4.
    pub fn validate(&self) -> AppResult<()> {
        let mut fields: HashMap<String, String> = HashMap::new();

        if self.version != SCHEMA_VERSION {
            fields.insert(
                "guest_state.version".into(),
                format!(
                    "Unsupported schema version {} — backend accepts {}.",
                    self.version, SCHEMA_VERSION
                ),
            );
            // No point continuing — the rest of the shape may have changed
            // across versions.
            return Err(AppError::Validation(fields));
        }

        if self.streak_count < 0 {
            fields.insert(
                "guest_state.streak_count".into(),
                "Must be non-negative.".into(),
            );
        }

        // Settings entries.
        for (key, s) in &self.settings {
            let cn = match parse_case_key(key) {
                Ok(n) => n,
                Err(msg) => {
                    fields.insert(format!("guest_state.settings.{key}"), msg);
                    continue;
                }
            };
            if let Some(rcn) = s.result_case_number {
                if !(1..=57).contains(&rcn) {
                    fields.insert(
                        format!("guest_state.settings.{cn}.result_case_number"),
                        "Must be between 1 and 57.".into(),
                    );
                }
            }
            if let Some(rot) = s.result_rotation {
                if !(0..=3).contains(&rot) {
                    fields.insert(
                        format!("guest_state.settings.{cn}.result_rotation"),
                        "Must be 0, 1, 2, or 3.".into(),
                    );
                }
            }
            // Tag-length cap mirrors normalize_tags. Cardinality caps run
            // post-normalization in the import path.
            for tag in &s.tags {
                if tag.chars().count() > TAG_MAX_LEN {
                    fields.insert(
                        format!("guest_state.settings.{cn}.tags"),
                        format!("Each tag must be {TAG_MAX_LEN} characters or fewer."),
                    );
                    break;
                }
            }
        }

        // Progress entries.
        for (key, p) in &self.progress {
            let cn = match parse_case_key(key) {
                Ok(n) => n,
                Err(msg) => {
                    fields.insert(format!("guest_state.progress.{key}"), msg);
                    continue;
                }
            };
            if !(EASE_FACTOR_MIN..=EASE_FACTOR_MAX).contains(&p.ease_factor) {
                fields.insert(
                    format!("guest_state.progress.{cn}.ease_factor"),
                    format!("Must be between {EASE_FACTOR_MIN} and {EASE_FACTOR_MAX}."),
                );
            }
            // The DB CHECK is `interval_days >= 1`. A row with no review yet
            // shouldn't be in the progress dict at all, so 0 here is malformed.
            if p.interval_days < 1 {
                fields.insert(
                    format!("guest_state.progress.{cn}.interval_days"),
                    "Must be at least 1.".into(),
                );
            }
            if p.repetitions < 0 {
                fields.insert(
                    format!("guest_state.progress.{cn}.repetitions"),
                    "Must be non-negative.".into(),
                );
            }
            if let Some(g) = p.last_grade {
                if !(0..=3).contains(&g) {
                    fields.insert(
                        format!("guest_state.progress.{cn}.last_grade"),
                        "Must be 0, 1, 2, or 3.".into(),
                    );
                }
            }
        }

        if fields.is_empty() {
            Ok(())
        } else {
            Err(AppError::Validation(fields))
        }
    }

    /// Import the validated guest state into the (just-created) user's rows.
    /// Caller must have already invoked `validate()`. Runs every write
    /// inside the supplied transaction so a partial failure rolls back.
    ///
    /// Tag-cap enforcement happens after settings normalization: we tally
    /// distinct + total counts across the whole import and reject the
    /// entire blob if either cap is exceeded. The frontend should never
    /// produce an over-cap blob (caps are enforced symmetrically there),
    /// but the backend treats it as a defense-in-depth check.
    pub async fn import(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
    ) -> AppResult<()> {
        // Build case_number → case_id map once per import.
        let case_rows: Vec<(i32, Uuid)> =
            sqlx::query_as("SELECT case_number, id FROM cases")
                .fetch_all(&mut **tx)
                .await?;
        let case_index: HashMap<i32, Uuid> = case_rows.into_iter().collect();

        // ─── Settings ──────────────────────────────────────────────────
        let mut total_links: usize = 0;
        let mut distinct: HashSet<String> = HashSet::new();

        for (key, s) in &self.settings {
            let case_number = parse_case_key(key)
                .map_err(|msg| validation_err("guest_state.settings", &msg))?;
            let case_id = *case_index.get(&case_number).ok_or_else(|| {
                validation_err(
                    "guest_state.settings",
                    &format!("Unknown case_number {case_number}."),
                )
            })?;

            let result_case_id = match s.result_case_number {
                None => None,
                Some(rcn) => Some(*case_index.get(&rcn).ok_or_else(|| {
                    validation_err(
                        "guest_state.settings.result_case_number",
                        &format!("Unknown result case_number {rcn}."),
                    )
                })?),
            };

            // Trim strings the same way the PATCH route does. Empty after
            // trim → clear (None) so we don't store whitespace-only strings.
            let nickname = trim_or_none(&s.nickname);
            let algorithm = trim_or_none(&s.algorithm);

            let tags = normalize_tags(s.tags.clone())
                .map_err(|msg| validation_err("guest_state.settings.tags", &msg))?;
            total_links += tags.len();
            for t in &tags {
                distinct.insert(t.clone());
            }

            // Skip rows that would resolve to all-NULL — same convention
            // as cases::update_settings.
            if nickname.is_none()
                && algorithm.is_none()
                && result_case_id.is_none()
                && s.result_rotation.is_none()
                && tags.is_empty()
            {
                continue;
            }

            sqlx::query(
                r#"
                INSERT INTO user_case_settings
                    (user_id, case_id, nickname, algorithm, result_case_id, result_rotation, tags)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(user_id)
            .bind(case_id)
            .bind(&nickname)
            .bind(&algorithm)
            .bind(result_case_id)
            .bind(s.result_rotation)
            .bind(if tags.is_empty() { None } else { Some(&tags) })
            .execute(&mut **tx)
            .await?;
        }

        if total_links > MAX_TAG_LINKS_PER_USER {
            return Err(validation_err(
                "guest_state.settings.tags",
                &format!(
                    "Total tag-links ({total_links}) exceeds the per-user limit of {MAX_TAG_LINKS_PER_USER}."
                ),
            ));
        }
        if distinct.len() > MAX_DISTINCT_TAGS_PER_USER {
            return Err(validation_err(
                "guest_state.settings.tags",
                &format!(
                    "Distinct tags ({}) exceeds the per-user limit of {MAX_DISTINCT_TAGS_PER_USER}.",
                    distinct.len(),
                ),
            ));
        }

        // ─── Progress ──────────────────────────────────────────────────
        for (key, p) in &self.progress {
            let case_number = parse_case_key(key)
                .map_err(|msg| validation_err("guest_state.progress", &msg))?;
            let case_id = *case_index.get(&case_number).ok_or_else(|| {
                validation_err(
                    "guest_state.progress",
                    &format!("Unknown case_number {case_number}."),
                )
            })?;

            sqlx::query(
                r#"
                INSERT INTO user_case_progress
                    (user_id, case_id, ease_factor, interval_days, repetitions,
                     due_date, last_grade, last_reviewed)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                "#,
            )
            .bind(user_id)
            .bind(case_id)
            .bind(p.ease_factor)
            .bind(p.interval_days)
            .bind(p.repetitions)
            .bind(p.due_date)
            .bind(p.last_grade)
            .bind(p.last_reviewed)
            .execute(&mut **tx)
            .await?;
        }

        // ─── Streak + onboarding ───────────────────────────────────────
        sqlx::query(
            r#"
            UPDATE users
            SET streak_count = $2,
                last_practice_date = $3,
                has_seen_onboarding = (has_seen_onboarding OR $4)
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(self.streak_count)
        .bind(self.last_practice_date)
        .bind(self.onboarding_completed)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    /// Merge the validated guest state into an existing authed user's
    /// rows. Caller must have already invoked `validate()`. Same
    /// transaction discipline as `import` — every write goes through `tx`
    /// so a partial failure rolls back. Returns counts of cases and
    /// distinct tags actually merged for the response payload.
    ///
    /// Merge rules — see docs/milestones/06_guest_mode.md §4:
    ///   - Settings: skip when a server override already exists for the
    ///     case_id. Server values are treated as authoritative.
    ///   - Progress: keep whichever side has the higher `interval_days`.
    ///     Ties → server. No server row → insert from guest.
    ///   - Streak: `MAX(server, guest)`. `last_practice_date` is the later
    ///     of the two (None counts as oldest).
    ///   - Onboarding: OR with existing flag.
    ///   - Tag caps: enforced post-merge across the union of server's
    ///     existing tags and any new tags inserted from the guest.
    pub async fn merge(
        &self,
        tx: &mut Transaction<'_, Postgres>,
        user_id: Uuid,
    ) -> AppResult<MergeSummary> {
        let case_rows: Vec<(i32, Uuid)> =
            sqlx::query_as("SELECT case_number, id FROM cases")
                .fetch_all(&mut **tx)
                .await?;
        let case_index: HashMap<i32, Uuid> = case_rows.into_iter().collect();

        // Snapshot existing override rows so we can decide which guest
        // entries to skip.
        let existing_settings: Vec<(Uuid, Option<Vec<String>>)> = sqlx::query_as(
            "SELECT case_id, tags FROM user_case_settings WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&mut **tx)
        .await?;
        let existing_case_ids: HashSet<Uuid> =
            existing_settings.iter().map(|(c, _)| *c).collect();

        // ─── Settings ──────────────────────────────────────────────────
        let mut total_links: usize = 0;
        let mut distinct: HashSet<String> = HashSet::new();
        for (_, tags) in &existing_settings {
            if let Some(tags) = tags {
                total_links += tags.len();
                for t in tags {
                    distinct.insert(t.clone());
                }
            }
        }

        let mut cases_merged = 0usize;
        for (key, s) in &self.settings {
            let case_number = parse_case_key(key)
                .map_err(|msg| validation_err("guest_state.settings", &msg))?;
            let case_id = *case_index.get(&case_number).ok_or_else(|| {
                validation_err(
                    "guest_state.settings",
                    &format!("Unknown case_number {case_number}."),
                )
            })?;

            if existing_case_ids.contains(&case_id) {
                // Server already has an override — never overwrite.
                continue;
            }

            let result_case_id = match s.result_case_number {
                None => None,
                Some(rcn) => Some(*case_index.get(&rcn).ok_or_else(|| {
                    validation_err(
                        "guest_state.settings.result_case_number",
                        &format!("Unknown result case_number {rcn}."),
                    )
                })?),
            };

            let nickname = trim_or_none(&s.nickname);
            let algorithm = trim_or_none(&s.algorithm);
            let tags = normalize_tags(s.tags.clone())
                .map_err(|msg| validation_err("guest_state.settings.tags", &msg))?;

            if nickname.is_none()
                && algorithm.is_none()
                && result_case_id.is_none()
                && s.result_rotation.is_none()
                && tags.is_empty()
            {
                continue;
            }

            total_links += tags.len();
            for t in &tags {
                distinct.insert(t.clone());
            }

            sqlx::query(
                r#"
                INSERT INTO user_case_settings
                    (user_id, case_id, nickname, algorithm, result_case_id, result_rotation, tags)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(user_id)
            .bind(case_id)
            .bind(&nickname)
            .bind(&algorithm)
            .bind(result_case_id)
            .bind(s.result_rotation)
            .bind(if tags.is_empty() { None } else { Some(&tags) })
            .execute(&mut **tx)
            .await?;
            cases_merged += 1;
        }

        if total_links > MAX_TAG_LINKS_PER_USER {
            return Err(validation_err(
                "guest_state.settings.tags",
                &format!(
                    "Total tag-links ({total_links}) exceeds the per-user limit of {MAX_TAG_LINKS_PER_USER}."
                ),
            ));
        }
        if distinct.len() > MAX_DISTINCT_TAGS_PER_USER {
            return Err(validation_err(
                "guest_state.settings.tags",
                &format!(
                    "Distinct tags ({}) exceeds the per-user limit of {MAX_DISTINCT_TAGS_PER_USER}.",
                    distinct.len(),
                ),
            ));
        }

        // ─── Progress ──────────────────────────────────────────────────
        let existing_progress: Vec<(Uuid, i32)> = sqlx::query_as(
            "SELECT case_id, interval_days FROM user_case_progress WHERE user_id = $1",
        )
        .bind(user_id)
        .fetch_all(&mut **tx)
        .await?;
        let existing_progress_idx: HashMap<Uuid, i32> =
            existing_progress.into_iter().collect();

        for (key, p) in &self.progress {
            let case_number = parse_case_key(key)
                .map_err(|msg| validation_err("guest_state.progress", &msg))?;
            let case_id = *case_index.get(&case_number).ok_or_else(|| {
                validation_err(
                    "guest_state.progress",
                    &format!("Unknown case_number {case_number}."),
                )
            })?;

            match existing_progress_idx.get(&case_id) {
                None => {
                    sqlx::query(
                        r#"
                        INSERT INTO user_case_progress
                            (user_id, case_id, ease_factor, interval_days, repetitions,
                             due_date, last_grade, last_reviewed)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
                        "#,
                    )
                    .bind(user_id)
                    .bind(case_id)
                    .bind(p.ease_factor)
                    .bind(p.interval_days)
                    .bind(p.repetitions)
                    .bind(p.due_date)
                    .bind(p.last_grade)
                    .bind(p.last_reviewed)
                    .execute(&mut **tx)
                    .await?;
                }
                Some(server_interval) if p.interval_days > *server_interval => {
                    sqlx::query(
                        r#"
                        UPDATE user_case_progress
                        SET ease_factor   = $3,
                            interval_days = $4,
                            repetitions   = $5,
                            due_date      = $6,
                            last_grade    = $7,
                            last_reviewed = $8
                        WHERE user_id = $1 AND case_id = $2
                        "#,
                    )
                    .bind(user_id)
                    .bind(case_id)
                    .bind(p.ease_factor)
                    .bind(p.interval_days)
                    .bind(p.repetitions)
                    .bind(p.due_date)
                    .bind(p.last_grade)
                    .bind(p.last_reviewed)
                    .execute(&mut **tx)
                    .await?;
                }
                // Server interval >= guest interval (including ties): keep server.
                _ => {}
            }
        }

        // ─── Streak + onboarding ───────────────────────────────────────
        sqlx::query(
            r#"
            UPDATE users
            SET streak_count = GREATEST(streak_count, $2),
                last_practice_date = CASE
                    WHEN last_practice_date IS NULL THEN $3
                    WHEN $3 IS NULL THEN last_practice_date
                    WHEN $3 > last_practice_date THEN $3
                    ELSE last_practice_date
                END,
                has_seen_onboarding = (has_seen_onboarding OR $4)
            WHERE id = $1
            "#,
        )
        .bind(user_id)
        .bind(self.streak_count)
        .bind(self.last_practice_date)
        .bind(self.onboarding_completed)
        .execute(&mut **tx)
        .await?;

        Ok(MergeSummary {
            cases: cases_merged,
            tags: distinct.len(),
        })
    }
}

/// Counts surfaced in the `/auth/merge-guest-state` response.
#[derive(Debug, serde::Serialize)]
pub struct MergeSummary {
    pub cases: usize,
    pub tags: usize,
}

/// Parse a case_number key from the JSON dict — string in `[1..=57]`.
fn parse_case_key(key: &str) -> Result<i32, String> {
    let n: i32 = key
        .parse()
        .map_err(|_| format!("Key '{key}' is not a valid case number."))?;
    if !(1..=57).contains(&n) {
        return Err(format!("Case number {n} out of range — must be 1–57."));
    }
    Ok(n)
}

/// Trim a string and coerce empty-after-trim to `None`. Mirrors
/// `routes::cases::trim_optional` semantics (the PATCH route's clear path).
fn trim_or_none(raw: &Option<String>) -> Option<String> {
    raw.as_ref().and_then(|s| {
        let t = s.trim();
        if t.is_empty() { None } else { Some(t.to_string()) }
    })
}

fn validation_err(key: &str, msg: &str) -> AppError {
    let mut fields = HashMap::new();
    fields.insert(key.to_string(), msg.to_string());
    AppError::Validation(fields)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_state() -> GuestState {
        GuestState {
            version: SCHEMA_VERSION,
            display_name: None,
            created_at: None,
            streak_count: 0,
            last_practice_date: None,
            onboarding_completed: false,
            settings: HashMap::new(),
            progress: HashMap::new(),
        }
    }

    #[test]
    fn validate_accepts_empty_state() {
        empty_state().validate().expect("empty is valid");
    }

    #[test]
    fn validate_rejects_wrong_version() {
        let mut s = empty_state();
        s.version = 99;
        s.validate().expect_err("wrong version rejects");
    }

    #[test]
    fn validate_rejects_negative_streak() {
        let mut s = empty_state();
        s.streak_count = -1;
        s.validate().expect_err("negative streak rejects");
    }

    #[test]
    fn validate_rejects_out_of_range_case_key() {
        let mut s = empty_state();
        s.settings.insert(
            "58".into(),
            GuestSettings {
                nickname: None,
                algorithm: None,
                result_case_number: None,
                result_rotation: None,
                tags: vec![],
            },
        );
        s.validate().expect_err("case 58 rejects");
    }

    #[test]
    fn validate_rejects_bad_ease_factor() {
        let mut s = empty_state();
        s.progress.insert(
            "1".into(),
            GuestProgress {
                ease_factor: 0.5, // below MIN
                interval_days: 1,
                repetitions: 0,
                due_date: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
                last_grade: None,
                last_reviewed: None,
            },
        );
        s.validate().expect_err("ease 0.5 rejects");
    }

    #[test]
    fn validate_rejects_zero_interval() {
        let mut s = empty_state();
        s.progress.insert(
            "1".into(),
            GuestProgress {
                ease_factor: 2.5,
                interval_days: 0,
                repetitions: 0,
                due_date: NaiveDate::from_ymd_opt(2026, 5, 1).unwrap(),
                last_grade: None,
                last_reviewed: None,
            },
        );
        s.validate().expect_err("interval 0 rejects (db checks for >= 1)");
    }

    #[test]
    fn validate_rejects_over_length_tag() {
        let mut s = empty_state();
        s.settings.insert(
            "1".into(),
            GuestSettings {
                nickname: None,
                algorithm: None,
                result_case_number: None,
                result_rotation: None,
                tags: vec!["a".repeat(TAG_MAX_LEN + 1)],
            },
        );
        s.validate().expect_err("over-cap tag rejects");
    }
}
