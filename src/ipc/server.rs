use std::{path::Path, sync::Arc};

use tokio::io::{AsyncBufReadExt, AsyncWriteExt};
use tokio::sync::Mutex;

use crate::error;
use crate::info;
use crate::{
  core::SubstrateCore,
  error::AppResult,
  ipc::protocol::{IPCRequest, IPCResponse},
  ipc::service::handle_request,
  util::{Config, get_storage_path},
};

pub async fn run(core: Arc<Mutex<SubstrateCore>>, _config: Config) -> AppResult<()> {
  let socket_path = get_storage_path("substrate.socket");
  let socket_path = Path::new(&socket_path);

  if socket_path.exists() {
    std::fs::remove_file(socket_path)?;
  }

  let listener = tokio::net::UnixListener::bind(socket_path)?;

  info!("[IPC] Initialized");

  loop {
    let (stream, _) = listener.accept().await?;
    let session_core = core.clone();

    tokio::spawn(async move {
      if let Err(err) = handle_client(stream, session_core).await {
        error!("[IPC] Client error: {err}");
      }
    });
  }
}

async fn handle_client(
  stream: tokio::net::UnixStream,
  core: Arc<Mutex<SubstrateCore>>,
) -> AppResult<()> {
  let (reader, mut writer) = stream.into_split();
  let mut reader = tokio::io::BufReader::new(reader);
  let mut line = String::new();

  loop {
    line.clear();
    let bytes = reader.read_line(&mut line).await?;
    if bytes == 0 {
      break;
    }

    let request: IPCRequest = serde_json::from_str(line.trim_end())?;
    let response = handle_request(request, core.clone()).await;

    let response = serde_json::to_string(&response)?;
    writer.write_all(response.as_bytes()).await?;
    writer.write_all(b"\n").await?;
  }

  Ok(())
}
