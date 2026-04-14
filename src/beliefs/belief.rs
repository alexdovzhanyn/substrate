use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
  pub id: String,
  pub content: String,
  pub tags: Vec<String>,
  pub possible_queries: Vec<String>,
}

pub struct RankedBelief {
  pub id: String,
  pub content: String,
  pub tags: Vec<String>,
  pub possible_queries: Vec<String>,
  pub score: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BeliefProposal {
  pub content: String,
  pub tags: Vec<String>,
  pub possible_queries: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefDraft {
  pub id: String,
  pub content: String,
  pub tags: Vec<String>,
  pub possible_queries: Vec<String>,
  pub potential_conflicts: Vec<Belief>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BeliefCommitment {
  pub id: String,
  pub content: String,
  pub tags: Vec<String>,
  pub possible_queries: Vec<String>,
  pub conflicts: Vec<BeliefConflict>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BeliefConflict {
  pub conflicting_belief_id: String,
  pub conflict_reason: ConflictReason,
  pub missed_query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ConflictReason {
  Invalidates,
  Duplicate,
}
