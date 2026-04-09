#[derive(Debug, Clone)]
pub struct BeliefCommitment {
  pub id: String,
  pub content: String,
  pub tags: Vec<String>,
  pub possible_queries: Vec<String>,
  pub conflicts: Vec<BeliefConflict>,
}

#[derive(Debug, Clone)]
pub struct BeliefConflict {
  pub conflicting_belief_id: String,
  pub conflict_reason: ConflictReason,
  pub missed_query: Option<String>,
}

#[derive(Debug, Clone)]
enum ConflictReason {
  Invalidates,
  Duplicate,
}
