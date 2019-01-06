use failure::Fail;

#[macro_export]
macro_rules! log {
  ($($arg:tt)*) => { eprintln!("\x1b[1m[zshrug]\x1b[0m {}", format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! info {
  ($($arg:tt)*) => { log!(" \x1b[1;32minfo\x1b[0m {}", format_args!($($arg)*)) };
}

#[macro_export]
macro_rules! error {
  ($($arg:tt)*) => { log!("\x1b[1;31merror\x1b[0m {}", format_args!($($arg)*)) };
}

pub fn log_error(error: &dyn Fail) {
  let thread = std::thread::current();
  let name = thread.name().unwrap_or("<unnamed>");

  error!("error in thread '{}': {}", name, error);
  for cause in error.iter_causes() {
    error!("caused by: {}", cause);
  }

  let backtrace =
    error.backtrace().map(|b| b.to_string()).filter(|s| !s.is_empty());
  if let Some(backtrace) = backtrace {
    error!("{}", backtrace);
  } else {
    error!("note: Run with `RUST_BACKTRACE=1` for a backtrace.");
  }
}
