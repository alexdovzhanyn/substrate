use crate::error::AppResult;
use crate::util::Config;
use crate::{beliefs, semantic};
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Clone)]
pub struct AppState {
  pub semantic_index: Arc<Mutex<semantic::SemanticIndex>>,
  pub belief_store: Arc<Mutex<beliefs::BeliefStore>>,
}

impl AppState {
  pub async fn initialize(config: &Config) -> AppResult<Self> {
    let semantic_index = semantic::SemanticIndex::initialize(&config).await?;
    let belief_store = beliefs::BeliefStore::initialize(&config)?;

    Ok(Self {
      semantic_index: Arc::new(Mutex::new(semantic_index)),
      belief_store: Arc::new(Mutex::new(belief_store)),
    })
  }
}
