use std::sync::Arc;

use axum::{
  Json, Router,
  extract::{Query, State},
  http::StatusCode,
  routing::{get, post},
};

use serde::Deserialize;

use tokio::sync::Mutex;

use crate::core::SubstrateCore;

#[derive(Clone)]
pub struct ApiState {
  pub core: Arc<Mutex<SubstrateCore>>,
}

pub fn router(core: Arc<Mutex<SubstrateCore>>) -> Router {
  let state = ApiState { core };

  Router::new()
    .route("/health", get(health))
    .route("/beliefs", get(list_beliefs))
    .with_state(state)
}

async fn health() -> &'static str {
  "ok"
}

#[derive(Deserialize)]
struct ListBeliefsParams {
  limit: Option<usize>,
  search: Option<String>,
  offset: Option<usize>,
}

async fn list_beliefs(
  State(state): State<ApiState>,
  Query(params): Query<ListBeliefsParams>,
) -> Result<Json<serde_json::Value>, (StatusCode, String)> {
  let beliefs = state
    .core
    .lock()
    .await
    .belief_store
    .get_beliefs(params.limit.unwrap_or(50), params.search, params.offset)
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))?;

  Ok(Json(serde_json::json!({
    "beliefs": beliefs
  })))
}
