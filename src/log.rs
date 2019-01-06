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
