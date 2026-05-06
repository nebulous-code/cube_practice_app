//! Case routes. See docs/milestones/02_case_data_and_browser.md §4.

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    routing::{get, patch},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::cases::{self, normalize_tags, Case, SettingsPatch};
use crate::error::{AppError, AppResult};
use crate::state::AppState;

const NICKNAME_MAX: usize = 80;
const ALGORITHM_MAX: usize = 1000;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/cases", get(list))
        .route("/cases/:id", get(detail))
        .route("/cases/:id/settings", patch(update_settings))
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    cases: Vec<Case>,
}

async fn list(
    State(state): State<AppState>,
    user: Option<AuthUser>,
) -> AppResult<Json<ListResponse>> {
    let cases = match user {
        Some(u) => cases::list_for_user(&state.pool, u.user_id).await?,
        None => cases::list_global(&state.pool).await?,
    };
    Ok(Json(ListResponse { cases }))
}

async fn detail(
    State(state): State<AppState>,
    user: Option<AuthUser>,
    Path(id): Path<Uuid>,
) -> AppResult<Json<Case>> {
    let case = match user {
        Some(u) => cases::get_for_user(&state.pool, u.user_id, id).await?,
        None => cases::get_global(&state.pool, id).await?,
    };
    Ok(Json(case))
}

/// `PATCH /cases/:id/settings` body. Each field uses `Option<Option<T>>` so
/// we can distinguish three cases:
///   - field absent (`None`): leave the existing override value untouched
///   - field is JSON `null` (`Some(None)`): clear the override (revert to default)
///   - field has a value (`Some(Some(v))`): set the override to `v`
#[derive(Debug, Deserialize, Default)]
pub struct UpdateSettingsRequest {
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    nickname: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    algorithm: Option<Option<String>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    result_case_id: Option<Option<Uuid>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    result_rotation: Option<Option<i32>>,
    #[serde(default, deserialize_with = "deserialize_optional_field")]
    tags: Option<Option<Vec<String>>>,
}

fn deserialize_optional_field<'de, T, D>(
    deserializer: D,
) -> Result<Option<Option<T>>, D::Error>
where
    T: serde::Deserialize<'de>,
    D: serde::Deserializer<'de>,
{
    Ok(Some(Option::<T>::deserialize(deserializer)?))
}

async fn update_settings(
    State(state): State<AppState>,
    user: AuthUser,
    Path(case_id): Path<Uuid>,
    Json(req): Json<UpdateSettingsRequest>,
) -> AppResult<Json<Case>> {
    let mut fields: HashMap<String, String> = HashMap::new();

    // Trim/validate strings up front. Trimmed-empty becomes None (clears).
    let nickname = trim_optional(&req.nickname, "nickname", NICKNAME_MAX, &mut fields);
    let algorithm = trim_optional(&req.algorithm, "algorithm", ALGORITHM_MAX, &mut fields);

    // Normalize tags. An empty post-normalization vector is coerced to
    // Some(None) (clears the override) — see milestone 04 §3.
    let tags = match req.tags {
        None => None,
        Some(None) => Some(None),
        Some(Some(raw)) => match normalize_tags(raw) {
            Ok(v) if v.is_empty() => Some(None),
            Ok(v) => Some(Some(v)),
            Err(msg) => {
                fields.insert("tags".into(), msg);
                None
            }
        },
    };

    if let Some(Some(rot)) = req.result_rotation {
        if !(0..=3).contains(&rot) {
            fields.insert(
                "result_rotation".into(),
                "Must be 0, 1, 2, or 3.".into(),
            );
        }
    }

    if !fields.is_empty() {
        return Err(AppError::Validation(fields));
    }

    let patch = SettingsPatch {
        nickname,
        algorithm,
        result_case_id: req.result_case_id,
        result_rotation: req.result_rotation,
        tags,
    };

    let case = cases::update_settings(&state.pool, user.user_id, case_id, patch).await?;
    Ok(Json(case))
}

/// Apply the trim+length rules to a string field. An empty trimmed string
/// is treated as `Some(None)` — explicit clear.
fn trim_optional(
    raw: &Option<Option<String>>,
    field: &str,
    max: usize,
    errors: &mut HashMap<String, String>,
) -> Option<Option<String>> {
    match raw {
        None => None,
        Some(None) => Some(None),
        Some(Some(s)) => {
            let trimmed = s.trim();
            if trimmed.is_empty() {
                Some(None)
            } else if trimmed.chars().count() > max {
                errors.insert(field.into(), format!("Must be {max} characters or fewer."));
                None
            } else {
                Some(Some(trimmed.to_string()))
            }
        }
    }
}
