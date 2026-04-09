use serde::Deserialize;
use std::fs;

use crate::error::AppResult;

#[derive(Debug, Deserialize)]
pub struct Config {
    pub retrieval: RetrievalConfig,
    pub storage: StorageConfig,
}

#[derive(Debug, Deserialize)]
pub struct RetrievalConfig {
    pub semantic_top_k: usize,
    pub max_l2_distance: f32,
    pub min_reranker_score: f32,
}

#[derive(Debug, Deserialize)]
pub struct StorageConfig {
    pub lancedb_file: String,
    pub redb_file: String,
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
