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
  configure_onnx_runtime();

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

pub fn configure_onnx_runtime() {
  if std::env::var_os("ORT_DYLIB_PATH").is_some() {
    return;
  }

  let Some(exe_dir) = std::env::current_exe()
    .ok()
    .and_then(|path| path.parent().map(|parent| parent.to_path_buf()))
  else {
    return;
  };

  let ort_path = exe_dir.join("vendor/onnxruntime/lib/libonnxruntime.dylib");

  if ort_path.exists() {
    unsafe {
      std::env::set_var("ORT_DYLIB_PATH", ort_path);
    }
  }
}
