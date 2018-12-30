use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
  pub fn init(root: PathBuf) -> Result<Self, Error> {
    fs::create_dir_all(&root).with_context(|_| {
      format!("couldn't create storage directory '{}'", root.display())
    })?;

    let state_path = root.join("state");
    let state =
      State::load(state_path).context("couldn't read storage state")?;

    Ok(Self { root, state })
  }

  pub fn download_plugin(&mut self, plugin: &Plugin) -> Result<(), Error> {
    if plugin.from == PluginSource::Local {
      return Ok(());
    }

    if self.state.is_plugin_downloaded(plugin) {
      return Ok(());
    }

    let plugin_dir = self.root.join(plugin.id());

    if plugin_dir.is_dir() {
      fs::remove_dir_all(&plugin_dir).with_context(|_| {
        format!("couldn't clear plugin directory '{}'", plugin_dir.display())
      })?;
    }

    fs::create_dir_all(&plugin_dir).with_context(|_| {
      format!(
        "couldn't create plugin directory '{}'",
        plugin_dir.display()
      )
    })?;

    match plugin.from {
      PluginSource::Git => clone_git_repository(&plugin.name, &plugin_dir),
      PluginSource::Url => download_file(&plugin.name, &plugin_dir),
      _ => unreachable!(),
    }
    .with_context(|_| format!("couldn't download plugin '{}'", plugin.name))?;

    self.state.add_downloaded_plugin(plugin)?;

    log!();

    Ok(())
  }
}

fn clone_git_repository(repo: &str, dir: &Path) -> Result<(), Error> {
  log!("Cloning git repository '{}'...", repo);

  let exit_status = Command::new("git")
    .arg("clone")
    .arg(repo)
    .arg(dir)
    .status()
    .context("couldn't run git")?;

  ensure!(exit_status.success(), "git has exited with an error");
  Ok(())
}

fn download_file(url: &str, dir: &Path) -> Result<(), Error> {
  log!("Downloading '{}'...", url);

  let exit_status = Command::new("wget")
    .arg("--directory-prefix")
    .arg(dir)
    .arg(url)
    .status()
    .context("couldn't run wget")?;

  ensure!(exit_status.success(), "git has exited with an error");
  Ok(())
}
