//! Progress routes. See docs/milestones/04_dashboard_progress_free_study_tags.md §4.

use std::collections::HashMap;

use axum::{
    extract::{Query, State},
    routing::get,
    Json, Router,
};
use serde::{Deserialize, Serialize};

use crate::auth::extractor::AuthUser;
use crate::cases::{Case, CaseState};
use crate::error::{AppError, AppResult};
use crate::progress::{self, ProgressSummary};
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/progress", get(summary))
        .route("/progress/cases", get(cases))
}

async fn summary(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ProgressSummary>> {
    let response = progress::summary_for_user(&state.pool, user.user_id).await?;
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct CasesQuery {
    state: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CasesResponse {
    cases: Vec<Case>,
}

async fn cases(
    State(state): State<AppState>,
    user: AuthUser,
    Query(query): Query<CasesQuery>,
) -> AppResult<Json<CasesResponse>> {
    let parsed_state = match query.state.as_deref() {
        None | Some("") => None,
        Some("not_started") => Some(CaseState::NotStarted),
        Some("learning") => Some(CaseState::Learning),
        Some("due") => Some(CaseState::Due),
        Some("mastered") => Some(CaseState::Mastered),
        Some(_) => {
            let mut fields = HashMap::new();
            fields.insert(
                "state".into(),
                "Must be one of: not_started, learning, due, mastered.".into(),
            );
            return Err(AppError::Validation(fields));
        }
    };

    let cases = progress::cases_for_user(&state.pool, user.user_id, parsed_state).await?;
    Ok(Json(CasesResponse { cases }))
}
