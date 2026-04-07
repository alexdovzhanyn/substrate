#[derive(Debug, Clone)]
pub struct BeliefProposal {
    pub subject: String,
    pub value: String,
    pub tags: Vec<String>,
    pub possible_queries: Vec<String>
}
