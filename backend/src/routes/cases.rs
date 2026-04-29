//! Case routes. See docs/milestones/02_case_data_and_browser.md §4.
//! C2 / C3 (`GET /cases/:id`, `PATCH /cases/:id/settings`) land in the next
//! chunk; this file currently only mounts `GET /cases`.

use axum::{extract::State, routing::get, Json, Router};
use serde::Serialize;

use crate::auth::extractor::AuthUser;
use crate::cases::{self, Case};
use crate::error::AppResult;
use crate::state::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/cases", get(list))
}

#[derive(Debug, Serialize)]
pub struct ListResponse {
    cases: Vec<Case>,
}

async fn list(
    State(state): State<AppState>,
    user: AuthUser,
) -> AppResult<Json<ListResponse>> {
    let cases = cases::list_for_user(&state.pool, user.user_id).await?;
    Ok(Json(ListResponse { cases }))
}
