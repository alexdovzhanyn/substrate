mod cli;
mod core;
mod error;
mod http;
mod ipc;
mod macros;
mod mcp;
mod tui;
mod util;

use crate::{error::AppResult, util::get_storage_path};

#[tokio::main]
async fn main() -> AppResult<()> {
  let args: Vec<String> = std::env::args().collect();

  ensure_config_exists()?;

  cli::route_command(args).await
}

fn ensure_config_exists() -> AppResult<()> {
  let config_file_path = get_storage_path("config.toml");

  if std::fs::exists(&config_file_path)? {
    return Ok(());
  }

  let default_config = include_str!("../assets/default_config.toml");

  std::fs::write(config_file_path, default_config)?;

  Ok(())
}
