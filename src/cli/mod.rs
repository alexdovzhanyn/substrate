use crate::error;
use crate::error::AppResult;
use crate::state::AppState;
use crate::util::Config;
use crate::util::logging;
use std::process::{Command, Stdio};

pub async fn route_command(args: Vec<String>) -> AppResult<()> {
  println!("{args:?}");

  match args.get(1).map(String::as_str) {
    Some("start") => command_start(&args).await?,
    Some("stop") => command_stop().await?,
    Some("serve") => command_serve().await?,
    _ => command_help().await?,
  }

  Ok(())
}

pub async fn command_start(args: &Vec<String>) -> AppResult<()> {
  let child = Command::new("substrate")
    .arg("serve")
    //.stdin(Stdio::null())
    //.stdout(Stdio::null())
    //.stderr(Stdio::null())
    .spawn()?;

  println!("PID is {}", child.id());

  Ok(())
}

pub async fn command_serve() -> AppResult<()> {
  let ascii_project_name = include_str!("../../assets/ascii_project_name.txt");
  println!("{}", ascii_project_name);

  let config = Config::load("config/default.toml")?;

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

pub async fn command_stop() -> AppResult<()> {
  Ok(())
}

pub async fn command_help() -> AppResult<()> {
  println!(
    "
\x1B[1mSubstrate v0.1.0\x1B[0m\n
\x1B[48;5;15;38;5;16;1m USAGE \x1B[0m\n
substrate [command] [options]

\x1B[48;5;15;38;5;16;1m COMMANDS \x1B[0m\n
  start                                  Start the Substrate daemon as a background process
  stop                                   Stop a running Substrate daemon

\x1B[48;5;15;38;5;16;1m OPTIONS \x1B[0m\n
  --foreground                           When passed with the start command, starts the daemon in the foreground instead
"
  );

  Ok(())
}
