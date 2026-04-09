use serde::Deserialize;
use std::fs;

use crate::error::AppResult;

#[derive(Debug, Deserialize, Clone)]
pub struct Config {
  pub retrieval: RetrievalConfig,
  pub storage: StorageConfig,
  pub mcp: McpConfig,
}

#[derive(Debug, Deserialize, Clone)]
pub struct RetrievalConfig {
  pub semantic_top_k: usize,
  pub max_l2_distance: f32,
  pub reranker_top_k: usize,
}

#[derive(Debug, Deserialize, Clone)]
pub struct StorageConfig {
  pub lancedb_file: String,
  pub redb_file: String,
}

#[derive(Debug, Deserialize, Clone)]
pub struct McpConfig {
  pub port: usize,
}

impl Config {
  pub fn load(path: &str) -> AppResult<Self> {
    let contents = fs::read_to_string(path)?;
    let config: Self = toml::from_str(&contents)?;

    Ok(config)
  }
}

pub fn get_storage_path(filename: &str) -> String {
  let app_support = dirs::data_local_dir()
    .expect("Unable to determine data directory")
    .join("Tesseract");

  fs::create_dir_all(&app_support)
    .expect("Unable to create Tesseract application support directory");

  app_support.join(filename).to_string_lossy().into_owned()
}
