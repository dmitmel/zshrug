#[macro_use]
extern crate failure;
extern crate failure_derive;

extern crate dirs;
extern crate md5;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

use failure::{Error, ResultExt};

macro_rules! log {
  ()            => { eprintln!() };
  ($($arg:tt)*) => { eprintln!("[zshrug] {}", format_args!($($arg)*)) };
}

mod config;
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

    log!("{}", error.backtrace());
    log!("note: Run with `RUST_BACKTRACE=1` if you don't see a backtrace.");

    process::exit(1);
  }
}

fn run() -> Result<(), Error> {
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

  for plugin in &config.plugins {
    storage.download_plugin(plugin)?;
  }

  Ok(())
}
