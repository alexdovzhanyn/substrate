use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Belief {
    pub id: String,
    pub subject: String,
    pub value: String,
    pub tags: Vec<String>,
    pub possible_queries: Vec<String>
}
