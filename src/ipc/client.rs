use tokio::{
  io::{AsyncBufReadExt, AsyncWriteExt, BufReader},
  net::{
    UnixStream,
    unix::{OwnedReadHalf, OwnedWriteHalf},
  },
};

use crate::{
  error::AppResult,
  ipc::protocol::{IPCRequest, IPCResponse},
  util::get_storage_path,
};

pub struct IPCClient {
  reader: BufReader<OwnedReadHalf>,
  writer: OwnedWriteHalf,
}

impl IPCClient {
  pub async fn connect() -> AppResult<Self> {
    let socket_path = get_storage_path("substrate.socket");

    let stream = UnixStream::connect(socket_path).await?;
    let (read_half, write_half) = stream.into_split();

    Ok(Self {
      reader: BufReader::new(read_half),
      writer: write_half,
    })
  }

  pub async fn request(&mut self, request: &IPCRequest) -> AppResult<IPCResponse> {
    self.send(request).await?;
    self.recv().await
  }

  async fn send(&mut self, request: &IPCRequest) -> AppResult<()> {
    let request = serde_json::to_string(request)?;

    self.writer.write_all(request.as_bytes()).await?;
    self.writer.write_all(b"\n").await?;

    Ok(())
  }

  async fn recv(&mut self) -> AppResult<IPCResponse> {
    let mut line = String::new();
    let bytes = self.reader.read_line(&mut line).await?;

    if bytes == 0 {
      return Err("[IPC] Server closed connection".into());
    }

    Ok(serde_json::from_str(line.trim_end())?)
  }
}
