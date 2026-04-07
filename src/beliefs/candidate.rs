use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateBelief {
    pub subject: String,
    pub value: String,
    pub score: f32
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CandidateBeliefEmbeddingEntry {
    pub belief_id: String,
    pub entry_id: String,
    pub embedding_source: String,
    pub embedding_text: String,
    pub score: f32
}
