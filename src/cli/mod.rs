use crate::beliefs::BeliefStore;
use crate::error;
use crate::error::AppResult;
use crate::semantic::SemanticIndex;
use crate::state::AppState;
use crate::util;
use crate::util::Config;
use crate::util::get_storage_path;
use crate::util::logging;
use std::fs::read_to_string;
use std::fs::remove_file;
use std::fs::write;
use std::fs::{OpenOptions, create_dir_all};
use std::io::Write;
use std::process::{Command, Stdio};

pub async fn route_command(args: Vec<String>) -> AppResult<()> {
  match args.get(1).map(String::as_str) {
    Some("start") => command_start(&args).await?,
    Some("stop") => command_stop().await?,
    Some("serve") => command_serve(&args).await?,
    Some("logs") => command_logs(&args).await?,
    Some("flush") => command_flush().await?,
    Some("status") => command_status().await?,
    _ => command_help().await?,
  }

  Ok(())
}

async fn command_start(args: &[String]) -> AppResult<()> {
  let existing_pid = get_process_pid()?;

  if existing_pid != 0 {
    println!("ERROR: Substrate is already running");
    return Ok(());
  }

  if args.iter().any(|arg| arg == "--foreground") {
    return command_serve(args).await;
  }

  let log = OpenOptions::new()
    .create(true)
    .append(true)
    .open(get_logging_path()?)?;

  let child = Command::new(std::env::current_exe()?)
    .arg("serve")
    .stdin(Stdio::null())
    .stdout(Stdio::from(log.try_clone()?))
    .stderr(Stdio::from(log))
    .spawn()?;

  let pid = child.id();

  println!(
    "Substrate starting as background process.\n + PID<{}>",
    &pid
  );

  write(get_storage_path(".pid"), pid.to_string())?;

  Ok(())
}

async fn command_serve(_args: &[String]) -> AppResult<()> {
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

async fn command_stop() -> AppResult<()> {
  let pid = get_process_pid()?;

  if pid != 0 {
    kill_process(pid)?;

    remove_file(get_storage_path(".pid"))?;

    let mut log = OpenOptions::new()
      .create(true)
      .append(true)
      .open(get_logging_path()?)?;

    writeln!(
      log,
      "{}",
      util::logging::get_log_content(util::logging::LogLevel::Info, "STOPPED".into())
    )?;

    println!("Stopped Substrate <{pid}>");
  }

  Ok(())
}

async fn command_flush() -> AppResult<()> {
  let pid = get_process_pid()?;

  if pid != 0 {
    println!("Can not flush Substrate data while running. Stop Substrate and try again");
    return Ok(());
  }

  let config = Config::load("config/default.toml")?;

  SemanticIndex::flush(&config).await?;
  BeliefStore::flush(&config)?;

  println!("Substrate data flushed");

  Ok(())
}

async fn command_logs(args: &[String]) -> AppResult<()> {
  let log_path = get_logging_path()?;

  if args.iter().any(|arg| arg == "--clear") {
    let _file = OpenOptions::new()
      .write(true)
      .truncate(true)
      .open(log_path)?;

    println!("Logs cleared.");

    return Ok(());
  }

  Command::new("tail")
    .arg("-n")
    .arg("100")
    .arg("-f")
    .arg(log_path)
    .status()?;

  Ok(())
}

async fn command_status() -> AppResult<()> {
  let pid = get_process_pid()?;

  if pid != 0 {
    println!("Substrate is \x1B[38;5;46;1m[Running]\x1B[0m");
    return Ok(());
  }

  println!("Substrate is \x1B[38;5;196;1m[Stopped]\x1B[0m");
  Ok(())
}

async fn command_help() -> AppResult<()> {
  println!(
    "
\x1B[1mSubstrate v0.1.0\x1B[0m\n
\x1B[48;5;15;38;5;16;1m USAGE \x1B[0m\n
substrate [command] [options]

\x1B[48;5;15;38;5;16;1m COMMANDS \x1B[0m\n
  start                                  Start the Substrate daemon as a background process
  stop                                   Stop a running Substrate daemon
  status                                 Check whether Substrate is currently running
  logs                                   View the logs produced by Substrate
  flush                                  Clears all Substrate belief data

\x1B[48;5;15;38;5;16;1m OPTIONS \x1B[0m\n
  --foreground                           When passed with the start command, starts the daemon in the foreground instead
  --clear                                When passed with the logs command, clears the log file
"
  );

  Ok(())
}

fn get_logging_path() -> AppResult<String> {
  let log_dir = util::get_storage_path("logs/");
  create_dir_all(&log_dir)?;

  Ok(log_dir + "substrate.log")
}

#[cfg(unix)]
fn kill_process(pid: u32) -> std::io::Result<()> {
  let result = unsafe { libc::kill(pid as i32, libc::SIGTERM) };

  if result == 0 {
    Ok(())
  } else {
    Err(std::io::Error::last_os_error())
  }
}

#[cfg(windows)]
fn kill_process(pid: u32) -> std::io::Result<()> {
  std::process::Command::new("taskkill")
    .args(["/PID", &pid.to_string(), "/T"])
    .status()?
    .success()
    .then_some(())
    .ok_or_else(std::io::Error::last_os_error)
}

fn get_process_pid() -> AppResult<u32> {
  match read_to_string(get_storage_path(".pid")) {
    Ok(pid) => Ok(pid.trim().parse()?),
    Err(err) if err.kind() == std::io::ErrorKind::NotFound => Ok(0),
    Err(err) => Err(err.into()),
  }
}
