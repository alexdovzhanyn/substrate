use std::ops::Deref;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
  pub id: String,
  pub content: String,
  pub tags: Vec<String>,
  pub possible_queries: Vec<String>,
  pub created_at: u64,
  pub updated_at: u64,
}

pub struct RankedBelief {
  pub belief: Belief,
  pub score: f32,
}

impl Deref for RankedBelief {
  type Target = Belief;

  fn deref(&self) -> &Self::Target {
    &self.belief
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BeliefProposal {
  pub content: String,
  pub tags: Vec<String>,
  pub possible_queries: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct BeliefDraft {
  pub belief: Belief,
  pub potential_conflicts: Vec<Belief>,
}

impl Deref for BeliefDraft {
  type Target = Belief;

  fn deref(&self) -> &Self::Target {
    &self.belief
  }
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BeliefCommitment {
  pub draft_id: String,
  pub conflict_resolutions: Vec<BeliefConflictResolution>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct BeliefConflictResolution {
  pub conflicting_belief_id: String,
  pub action: ResolutionAction,
  pub missed_query: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub enum ResolutionAction {
  Invalidate,
  MergeDuplicate,
  Ignore,
}
