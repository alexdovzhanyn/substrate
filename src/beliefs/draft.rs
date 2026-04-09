use super::belief::Belief;

#[derive(Debug, Clone)]
pub struct BeliefDraft {
  pub id: String,
  pub content: String,
  pub tags: Vec<String>,
  pub possible_queries: Vec<String>,
  pub suggested_tags: Vec<String>,
  pub candidate_matches: Vec<Belief>,
}
