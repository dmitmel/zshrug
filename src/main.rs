extern crate failure;

extern crate cluFlock;
extern crate dirs;
extern crate globset;
extern crate md5;
extern crate walkdir;

extern crate serde;
extern crate serde_yaml;

use std::path::PathBuf;

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
  let storage_root =
    get_storage_root().context("couldn't get storage root directory")?;
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

fn get_storage_root() -> Fallible<PathBuf> {
  fn get_path_from_env(name: &str) -> Option<PathBuf> {
    use std::env;
    env::var_os(name).map(PathBuf::from)
  }

  Ok(if let Some(storage_root) = get_path_from_env("ZSHRUG_STORAGE_ROOT") {
    storage_root
  } else {
    // this is a simplified version of dirs::data_local_dir

    let home_dir = dirs::home_dir()
      .ok_or_else(|| format_err!("couldn't get your home directory"))?;

    let local_data_dir = get_path_from_env("XDG_DATA_HOME")
      .filter(|xdg_data_home| xdg_data_home.is_absolute())
      .unwrap_or_else(|| home_dir.join(".local").join("share"));

    local_data_dir.join("zshrug")
  })
}
