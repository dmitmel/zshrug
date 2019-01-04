extern crate failure;

extern crate cluFlock;
extern crate dirs;
extern crate globset;
extern crate md5;
extern crate walkdir;

extern crate serde;
extern crate serde_yaml;

use failure::*;

macro_rules! log {
  ()            => { eprintln!() };
  ($($arg:tt)*) => { eprintln!("\x1b[1m[zshrug]\x1b[0m {}", format_args!($($arg)*)) };
}

mod config;
mod script;
mod storage;

fn main() {
  if let Err(error) = run() {
    use std::{process, thread};

    let thread = thread::current();
    let name = thread.name().unwrap_or("<unnamed>");

    log!("error in thread '{}': {}", name, error);

    for cause in error.iter_causes() {
      log!("caused by: {}", cause);
    }

    let backtrace = error.backtrace().to_string();
    if backtrace.is_empty() {
      log!("note: Run with `RUST_BACKTRACE=1` for a backtrace.");
    } else {
      log!("{}", backtrace);
    }

    process::exit(1);
  }
}

fn run() -> Fallible<()> {
  let storage_root = dirs::cache_dir()
    .ok_or_else(|| format_err!("couldn't get system cache directory"))?
    .join(env!("CARGO_PKG_NAME"));
  let mut storage = storage::Storage::init(storage_root)
    .context("couldn't initialize storage")?;

  let input: String = {
    use std::io::{self, Read};

    let mut buffer = String::new();
    io::stdin()
      .read_to_string(&mut buffer)
      .context("couldn't read config from stdin")?;
    buffer
  };

  let config: config::Config =
    serde_yaml::from_str(&input).context("couldn't parse config")?;

  storage.ensure_plugins_installed(&config.plugins)?;

  let script = script::generate(&storage, &config.plugins)
    .context("couldn't generate script")?;
  println!("{}", script);

  Ok(())
}
