#[derive(Debug, Clone)]
pub struct BeliefProposal {
  pub content: String,
  pub tags: Vec<String>,
  pub possible_queries: Vec<String>,
}
