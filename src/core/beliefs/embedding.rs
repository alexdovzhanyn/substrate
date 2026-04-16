use std::sync::Arc;

use arrow_array::{ArrayRef, FixedSizeListArray, Float32Array, LargeStringArray, RecordBatch};
use arrow_schema::{DataType, Field, Schema};

use crate::error::AppResult;

#[derive(Debug, Clone)]
pub struct BeliefEmbeddingEntry {
  pub belief_id: String,
  pub entry_id: String,
  pub embedding_source: String,
  pub embedding_text: String,
  pub vector: Vec<f32>,
}

impl BeliefEmbeddingEntry {
  pub fn get_schema() -> Arc<Schema> {
    Arc::new(Schema::new(vec![
      Field::new("belief_id", DataType::LargeUtf8, false),
      Field::new("entry_id", DataType::LargeUtf8, false),
      Field::new("embedding_source", DataType::LargeUtf8, false),
      Field::new("embedding_text", DataType::LargeUtf8, false),
      Field::new(
        "vector",
        DataType::FixedSizeList(Arc::new(Field::new("item", DataType::Float32, false)), 384),
        false,
      ),
    ]))
  }

  pub fn to_record_batch(entries: &[BeliefEmbeddingEntry]) -> AppResult<RecordBatch> {
    const EMBEDDING_DIM: i32 = 384;

    for entry in entries {
      if entry.vector.len() != EMBEDDING_DIM as usize {
        return Err(
          format!(
            "expected embedding dimension {}, got {} for entry {}",
            EMBEDDING_DIM,
            entry.vector.len(),
            entry.entry_id
          )
          .into(),
        );
      }
    }

    let schema = Self::get_schema();

    let belief_id_array = Arc::new(LargeStringArray::from(
      entries
        .iter()
        .map(|e| e.belief_id.as_str())
        .collect::<Vec<_>>(),
    )) as ArrayRef;

    let entry_id_array = Arc::new(LargeStringArray::from(
      entries
        .iter()
        .map(|e| e.entry_id.as_str())
        .collect::<Vec<_>>(),
    )) as ArrayRef;

    let embedding_source_array = Arc::new(LargeStringArray::from(
      entries
        .iter()
        .map(|e| e.embedding_source.as_str())
        .collect::<Vec<_>>(),
    )) as ArrayRef;

    let embedding_text_array = Arc::new(LargeStringArray::from(
      entries
        .iter()
        .map(|e| e.embedding_text.as_str())
        .collect::<Vec<_>>(),
    )) as ArrayRef;

    let flat_vectors: Vec<f32> = entries
      .iter()
      .flat_map(|e| e.vector.iter().copied())
      .collect();

    let vector_values = Arc::new(Float32Array::from(flat_vectors)) as ArrayRef;

    let vector_array = Arc::new(FixedSizeListArray::try_new(
      Arc::new(Field::new("item", DataType::Float32, false)),
      EMBEDDING_DIM,
      vector_values,
      None,
    )?) as ArrayRef;

    Ok(RecordBatch::try_new(
      schema,
      vec![
        belief_id_array,
        entry_id_array,
        embedding_source_array,
        embedding_text_array,
        vector_array,
      ],
    )?)
  }
}
