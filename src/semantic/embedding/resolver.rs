use fastembed::{EmbeddingModel, Error, InitOptions, TextEmbedding};

use crate::error::AppResult;
use crate::info;

pub struct EmbeddingResolver {
  model: TextEmbedding,
}

impl EmbeddingResolver {
  pub fn initialize() -> AppResult<Self> {
    info!("[EmbeddingResolver] Initializing...");

    let model = TextEmbedding::try_new(
      InitOptions::new(EmbeddingModel::BGESmallENV15)
        .with_cache_dir(Self::get_model_path())
        .with_show_download_progress(false),
    )?;

    info!("[EmbeddingResolver] Initialized.");

    Ok(Self { model })
  }

  pub fn embed(&mut self, inputs: &[String]) -> Result<Vec<Vec<f32>>, Error> {
    let embeddings = self.model.embed(inputs, None)?;

    Ok(embeddings)
  }

  pub fn embed_single(&mut self, input: String) -> Result<Vec<f32>, Error> {
    let embeddings = self.model.embed(vec![&input], None)?;

    embeddings
      .into_iter()
      .next()
      .ok_or_else(|| Error::msg("No embedding returned"))
  }

  fn get_model_path() -> std::path::PathBuf {
    let model_dir = "models/bge-small-en-v1.5".to_string();

    std::env::current_exe()
      .unwrap()
      .parent()
      .unwrap()
      .join(model_dir)
  }
}
