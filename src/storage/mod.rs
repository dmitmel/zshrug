use cluFlock::{ExclusiveSliceLock, Flock};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use failure::*;

use crate::config::*;

mod state;
use self::state::State;

#[derive(Debug)]
pub struct Storage {
  root: PathBuf,
  state: State,
}

impl Storage {
  pub fn init(root: PathBuf) -> Fallible<Self> {
    fs::create_dir_all(&root).with_context(|_| {
      format!("couldn't create storage directory '{}'", root.display())
    })?;

    let state_path = root.join("state");
    let state = State::new(state_path);

    Ok(Self { root, state })
  }

  pub fn plugin_dir(&self, plugin: &Plugin) -> PathBuf {
    self.root.join("plugins").join(plugin.id())
  }

  fn is_plugin_downloaded(&self, plugin: &Plugin) -> Fallible<bool> {
    let result = self
      .state
      .is_plugin_downloaded(plugin)
      .context("couldn't read storage state")?;
    Ok(result)
  }

  pub fn ensure_plugin_downloaded(&mut self, plugin: &Plugin) -> Fallible<()> {
    if self.is_plugin_downloaded(plugin)? {
      return Ok(());
    }

    log!("downloading plugin {:?} from {:?}", plugin.name, plugin.from);

    let lock_path = self.root.join("lock");
    let lock_file = File::create(&lock_path)?;
    let (another_process_was_running, _lock) =
      exclusively_lock_file(&lock_file).with_context(|_| {
        format!("couldn't lock file '{}'", lock_path.display())
      })?;

    if another_process_was_running && self.is_plugin_downloaded(plugin)? {
      log!("another process has just downloaded this plugin");
      return Ok(());
    }

    let plugin_dir = self.plugin_dir(plugin);
    download_plugin(plugin, &plugin_dir).with_context(|_| {
      format!("couldn't download plugin {} from {:?}", plugin.name, plugin.from)
    })?;
    build_plugin(plugin, &plugin_dir).with_context(|_| {
      format!("couldn't build plugin {} from {:?}", plugin.name, plugin.from)
    })?;

    self
      .state
      .add_downloaded_plugin(plugin)
      .context("couldn't save storage state")?;

    Ok(())
  }
}

fn download_plugin(plugin: &Plugin, directory: &Path) -> Fallible<()> {
  // clear plugin directory to avoid conflicts and errors
  if directory.is_dir() {
    fs::remove_dir_all(&directory).with_context(|_| {
      format!("couldn't clear plugin directory '{}'", directory.display())
    })?;
  }

  fs::create_dir_all(&directory).with_context(|_| {
    format!("couldn't create plugin directory '{}'", directory.display())
  })?;

  match plugin.from {
    PluginSource::Git => clone_git_repository(&plugin.name, &directory)?,
    PluginSource::Url => download_file(&plugin.name, &directory)?,
    _ => unreachable!(),
  }

  Ok(())
}

fn exclusively_lock_file(
  file: &File,
) -> io::Result<(bool, ExclusiveSliceLock)> {
  file.try_exclusive_lock().and_then(|result| match result {
    Some(lock) => Ok((false, lock)),
    None => {
      log!("waiting for another process to unlock the lock file");
      file.exclusive_lock().map(|lock| (true, lock))
    }
  })
}

fn build_plugin(plugin: &Plugin, directory: &Path) -> Fallible<()> {
  if plugin.build.is_empty() {
    return Ok(());
  }

  log!("running build command: {}", plugin.build);

  let exit_status = Command::new("zsh")
    .arg("-c")
    .arg(&plugin.build)
    .current_dir(directory)
    .stdout(Stdio::null())
    .status()
    .context("couldn't run zsh")?;

  ensure!(exit_status.success(), "zsh has exited with an error");
  Ok(())
}

fn clone_git_repository(repo: &str, directory: &Path) -> Fallible<()> {
  log!("cloning git repository '{}'...", repo);

  let exit_status = Command::new("git")
    .arg("clone")
    .arg(repo)
    .arg(directory)
    .status()
    .context("couldn't run git")?;

  ensure!(exit_status.success(), "git has exited with an error");
  Ok(())
}

fn download_file(url: &str, directory: &Path) -> Fallible<()> {
  log!("downloading '{}'...", url);

  let exit_status = Command::new("wget")
    .arg("--directory-prefix")
    .arg(directory)
    .arg(url)
    .status()
    .context("couldn't run wget")?;

  ensure!(exit_status.success(), "wget has exited with an error");
  Ok(())
}
