use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

use failure::{Error, ResultExt};

use config::*;

pub struct Storage {
  pub root: PathBuf,
}

impl Storage {
  pub fn init(root: PathBuf) -> Result<Self, Error> {
    fs::create_dir_all(&root).with_context(|_| {
      format!("couldn't create storage directory '{}'", root.display())
    })?;

    Ok(Self { root })
  }

  pub fn download_plugin(&self, plugin: &Plugin) -> Result<(), Error> {
    if plugin.from == PluginSource::Local {
      return Ok(());
    }

    let plugin_dir = self.root.join(plugin.id());

    if plugin_dir.is_dir() {
      return Ok(());
    }

    fs::create_dir_all(&plugin_dir).with_context(|_| {
      format!(
        "couldn't create plugin directory '{}'",
        plugin_dir.display()
      )
    })?;

    let download_result: Result<(), _> = match plugin.from {
      PluginSource::Git => clone_git_repository(&plugin.name, &plugin_dir),
      PluginSource::Url => download_file(&plugin.name, &plugin_dir),
      _ => unreachable!(),
    };

    if download_result.is_err() {
      fs::remove_dir_all(&plugin_dir).unwrap();
      download_result.with_context(|_| {
        format!("couldn't download plugin '{}'", plugin.name)
      })?;
    }

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
