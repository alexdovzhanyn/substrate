mod cli;
mod core;
mod error;
mod ipc;
mod macros;
mod mcp;
mod tui;
mod util;

use crate::error::AppResult;

#[tokio::main]
async fn main() -> AppResult<()> {
  let args: Vec<String> = std::env::args().collect();

  cli::route_command(args).await
}
