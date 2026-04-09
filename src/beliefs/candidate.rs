use arrow_array::{Array, Float32Array, LargeStringArray, RecordBatch};
use serde::{Deserialize, Serialize};

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateBelief {
    pub subject: String,
    pub value: String,
    pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateBeliefEmbeddingEntry {
    pub belief_id: String,
    pub entry_id: String,
    pub embedding_source: String,
    pub embedding_text: String,
    pub score: f32,
}

impl CandidateBeliefEmbeddingEntry {
    pub fn from_record_batch_stream(stream: &Vec<RecordBatch>) -> AppResult<Vec<Self>> {
        let mut candidates: Vec<Self> = Vec::new();

        for batch in stream {
            let belief_id_col = batch
                .column_by_name("belief_id")
                .ok_or("Missing belief_id column")?
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or("belief_id column had wrong type")?;

            let entry_id_col = batch
                .column_by_name("entry_id")
                .ok_or("Missing entry_id column")?
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or("entry_id column had wrong type")?;

            let embedding_source_col = batch
                .column_by_name("embedding_source")
                .ok_or("Missing embedding_source column")?
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or("embedding_source column had wrong type")?;

            let embedding_text_col = batch
                .column_by_name("embedding_text")
                .ok_or("Missing embedding_text column")?
                .as_any()
                .downcast_ref::<LargeStringArray>()
                .ok_or("embedding_text column had wrong type")?;

            let distance_col = batch
                .column_by_name("_distance")
                .ok_or("Missing _distance column")?
                .as_any()
                .downcast_ref::<Float32Array>()
                .ok_or("_distance column had wrong type")?;

            for row_idx in 0..batch.num_rows() {
                if distance_col.is_null(row_idx) {
                    continue;
                }

                candidates.push(CandidateBeliefEmbeddingEntry {
                    belief_id: belief_id_col.value(row_idx).to_string(),
                    entry_id: entry_id_col.value(row_idx).to_string(),
                    embedding_source: embedding_source_col.value(row_idx).to_string(),
                    embedding_text: embedding_text_col.value(row_idx).to_string(),
                    score: distance_col.value(row_idx),
                });
            }
        }

        Ok(candidates)
    }
}
