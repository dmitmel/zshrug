use std::alloc::System;

#[global_allocator]
static A: System = System;

extern crate clap;
extern crate cluFlock;
extern crate dirs;
extern crate failure;
extern crate md5;
extern crate os_pipe;
extern crate serde;
extern crate serde_yaml;

use std::path::PathBuf;

use failure::*;

#[macro_use]
mod log;
mod cli;
mod config;
mod script;
mod storage;

fn main() {
  if let Err(error) = run() {
    log::log_error(error.as_ref());
    std::process::exit(1);
  }
}

fn run() -> Fallible<()> {
  use self::cli::{Command, Options};

  let options: Options = cli::parse_options();

  let storage_root =
    get_storage_root().context("couldn't get storage root directory")?;
  let mut storage = storage::Storage::init(storage_root.clone())
    .context("couldn't initialize storage")?;

  match options.command {
    Command::Init => {
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

      let plugins: Vec<&config::Plugin> = config.plugins.iter().collect();
      let installed_plugins = storage.ensure_plugins_are_installed(&plugins)?;

      let script = script::generate(&storage, &installed_plugins)
        .context("couldn't generate script")?;
      println!("{}", script);
    }

    Command::Storage => println!("{}", storage_root.display()),

    _ => bail!("command is not supported"),
  }

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
