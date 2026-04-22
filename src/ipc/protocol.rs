use serde::{Deserialize, Serialize};

use crate::core::beliefs::belief::Belief;

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum IPCRequest {
  ListBeliefs {
    search: Option<String>,
    limit: Option<usize>,
    offset: Option<usize>,
  },
  GetBelief {
    id: String,
  },
  SubscribeAccessLog,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(tag = "type")]
pub enum IPCResponse {
  ListBeliefs { beliefs: Vec<Belief> },
  Ok,
  Error { message: String },
}
