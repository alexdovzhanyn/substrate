use std::fmt;
use std::sync::OnceLock;

use chrono::Local;

use crate::util::Config;

static LOG_LEVEL: OnceLock<LogLevel> = OnceLock::new();

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum LogLevel {
  Error = 1,
  Warn = 2,
  Info = 3,
  Debug = 4,
  Trace = 5,
}

impl LogLevel {
  pub fn from_str(level: &str) -> Self {
    match level.to_lowercase().as_str() {
      "error" => LogLevel::Error,
      "warn" => LogLevel::Warn,
      "info" => LogLevel::Info,
      "debug" => LogLevel::Debug,
      "trace" => LogLevel::Trace,
      _ => LogLevel::Info,
    }
  }

  fn color_code(self) -> &'static str {
    match self {
      LogLevel::Error => "\x1b[31m", // red
      LogLevel::Warn => "\x1b[33m",  // yellow
      LogLevel::Info => "\x1b[32m",  // green
      LogLevel::Debug => "\x1b[34m", // blue
      LogLevel::Trace => "\x1b[35m", // magenta
    }
  }
}

impl fmt::Display for LogLevel {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let s = match self {
      LogLevel::Error => "ERROR",
      LogLevel::Warn => "WARN",
      LogLevel::Info => "INFO",
      LogLevel::Debug => "DEBUG",
      LogLevel::Trace => "TRACE",
    };
    write!(f, "{s}")
  }
}

pub fn init(config: &Config) {
  let level = LogLevel::from_str(&config.logging.level);
  let _ = LOG_LEVEL.set(level);
}

fn should_log(level: LogLevel) -> bool {
  let current = LOG_LEVEL.get().copied().unwrap_or(LogLevel::Info);
  level <= current
}

pub fn log(level: LogLevel, message: String) {
  if !should_log(level) {
    return;
  }

  println!("{}", get_log_content(level, message));
}

pub fn get_log_content(level: LogLevel, message: String) -> String {
  let timestamp = Local::now().format("%m-%d-%YT%H:%M:%S");

  // light gray for timestamp
  let ts_color = "\x1b[90m";
  let reset = "\x1b[0m";

  let level_color = level.color_code();

  format!("{ts_color}{timestamp}{reset} {level_color}[{level}]{reset} {message}")
}
