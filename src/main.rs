mod beliefs;
mod error;
mod mcp;
mod semantic;
mod state;
mod util;

use std::io::{self, Write};

use crate::error::AppResult;
use crate::state::AppState;

#[tokio::main]
async fn main() -> AppResult<()> {
  println!("Tesseract starting...");

  let config = util::Config::load("config/default.toml")?;

  let state = AppState::initialize(&config).await?;

  let mcp_state = state.clone();
  let mcp_config = config.clone();

  let mcp_handle = tokio::spawn(async move {
    if let Err(err) = crate::mcp::server::run(mcp_state, mcp_config).await {
      eprintln!("MCP server exited with error: {err}");
    }

    println!("MCP server started.");
  });

  let _ = mcp_handle.await;

  Ok(())
}
