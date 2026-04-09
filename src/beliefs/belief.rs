use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
  pub id: String,
  pub content: String,
  pub tags: Vec<String>,
  pub possible_queries: Vec<String>,
}
