//! Study routes. See docs/milestones/03_core_study_loop.md §5.

use std::collections::HashMap;

use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use chrono::Utc;
use serde::Deserialize;
use uuid::Uuid;

use crate::auth::extractor::AuthUser;
use crate::error::{AppError, AppResult};
use crate::srs::Grade;
use crate::state::AppState;
use crate::study::{self, DueResponse, ReviewResponse};

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/study/due", get(due))
        .route("/study/:case_id/review", post(review))
}

async fn due(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<DueResponse>> {
    let response = study::due_for_user(&state.pool, user.user_id).await?;
    Ok(Json(response))
}

#[derive(Debug, Deserialize)]
pub struct ReviewRequest {
    grade: u8,
}

async fn review(
    State(state): State<AppState>,
    user: AuthUser,
    Path(case_id): Path<Uuid>,
    Json(req): Json<ReviewRequest>,
) -> AppResult<Json<ReviewResponse>> {
    let grade = Grade::from_u8(req.grade).ok_or_else(|| {
        let mut fields = HashMap::new();
        fields.insert("grade".into(), "Must be 0, 1, 2, or 3.".into());
        AppError::Validation(fields)
    })?;

    let today = Utc::now().date_naive();
    let response = study::apply_review(&state.pool, user.user_id, case_id, grade, today).await?;
    Ok(Json(response))
}
