use fastembed::{EmbeddingModel, Error, InitOptions, TextEmbedding};

use crate::error::AppResult;

pub struct EmbeddingResolver {
  model: TextEmbedding,
}

impl EmbeddingResolver {
  pub fn initialize() -> AppResult<Self> {
    let model = TextEmbedding::try_new(
      InitOptions::new(EmbeddingModel::BGESmallENV15)
        .with_cache_dir(Self::get_model_path())
        .with_show_download_progress(false),
    )?;

    Ok(Self { model })
  }

  pub fn embed(&mut self, input: String) -> Result<Vec<Vec<f32>>, Error> {
    let embeddings = self.model.embed(vec![input], None)?;

    Ok(embeddings)
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
