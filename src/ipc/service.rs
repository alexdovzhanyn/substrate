use std::sync::Arc;

use tokio::sync::Mutex;

use crate::{
  core::SubstrateCore,
  ipc::protocol::{IPCRequest, IPCResponse},
};

pub async fn handle_request(request: IPCRequest, core: Arc<Mutex<SubstrateCore>>) -> IPCResponse {
  match request {
    IPCRequest::ListBeliefs {
      search,
      limit,
      offset,
    } => list_beliefs(search, limit, offset, &core).await,
    _ => IPCResponse::Error {
      message: String::from("Uknown action"),
    },
  }
}

pub async fn list_beliefs(
  search: Option<String>,
  limit: Option<usize>,
  offset: Option<usize>,
  core: &Arc<Mutex<SubstrateCore>>,
) -> IPCResponse {
  let res = core
    .lock()
    .await
    .belief_store
    .get_beliefs(limit.unwrap_or(50), search, offset);

  match res {
    Ok(beliefs) => IPCResponse::ListBeliefs { beliefs },
    Err(err) => to_ipc_err(err),
  }
}

fn to_ipc_err<E: std::fmt::Display>(e: E) -> IPCResponse {
  IPCResponse::Error {
    message: e.to_string(),
  }
}
