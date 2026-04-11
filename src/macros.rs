#[macro_export]
macro_rules! log {
  ($level:expr, $($arg:tt)*) => {
    $crate::util::logging::log($level, format!($($arg)*))
  };
}

#[macro_export]
macro_rules! error {
  ($($arg:tt)*) => {
    $crate::log!($crate::util::logging::LogLevel::Error, $($arg)*)
  };
}

#[macro_export]
macro_rules! warn {
  ($($arg:tt)*) => {
    $crate::log!($crate::util::logging::LogLevel::Warn, $($arg)*)
  };
}

#[macro_export]
macro_rules! info {
  ($($arg:tt)*) => {
    $crate::log!($crate::util::logging::LogLevel::Info, $($arg)*)
  };
}

#[macro_export]
macro_rules! debug {
  ($($arg:tt)*) => {
    $crate::log!($crate::util::logging::LogLevel::Debug, $($arg)*)
  };
}

#[macro_export]
macro_rules! trace {
  ($($arg:tt)*) => {
    $crate::log!($crate::util::logging::LogLevel::Trace, $($arg)*)
  };
}
