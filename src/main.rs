mod beliefs;
mod error;
mod macros;
mod mcp;
mod semantic;
mod state;
mod util;

use std::io::{self, Write};

use crate::error::AppResult;
use crate::state::AppState;
use crate::util::logging;

#[tokio::main]
async fn main() -> AppResult<()> {
  let ascii_project_name = include_str!("../assets/ascii_project_name.txt");
  println!("{}", ascii_project_name);

  let config = util::Config::load("config/default.toml")?;

  logging::init(&config);

  let state = AppState::initialize(&config).await?;

  let mcp_state = state.clone();
  let mcp_config = config.clone();

  let mcp_handle = tokio::spawn(async move {
    if let Err(err) = crate::mcp::server::run(mcp_state, mcp_config).await {
      error!("MCP server exited with error: {err}");
    }
  });

  let _ = mcp_handle.await;

  Ok(())
}
