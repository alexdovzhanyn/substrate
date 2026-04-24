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

    if let Ok(substrate_home) = std::env::var("SUBSTRATE_HOME") {
      return std::path::PathBuf::from(substrate_home).join(model_dir);
    }

    std::env::current_exe()
      .unwrap()
      .parent()
      .unwrap()
      .join(model_dir)
  }
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn loads_local_model_and_creates_embedding() {
    let mut resolver = EmbeddingResolver::initialize()
      .expect("embedding resolver should initialize using local model files");

    let embedding = resolver
      .embed_single("Substrate stores reusable context for AI agents.".to_string())
      .expect("embedding resolver should create an embedding");

    assert_eq!(
      embedding.len(),
      384,
      "BGE small English v1.5 should produce 384-dimensional embeddings"
    );

    assert!(
      embedding.iter().all(|value| value.is_finite()),
      "embedding should only contain finite values"
    );

    assert!(
      embedding.iter().any(|value| *value != 0.0),
      "embedding should not be all zeroes"
    );
  }
}
