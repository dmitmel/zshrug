use std::fs;
use std::path::Path;
use std::process::Command;

use failure::{Error, ResultExt};

use config::*;

pub fn download_plugins(config: &Config) -> Result<(), Error> {
  let storage_dir = dirs::cache_dir()
    .ok_or_else(|| format_err!("couldn't get system cache directory"))?
    .join(env!("CARGO_PKG_NAME"));

  fs::create_dir_all(&storage_dir).with_context(|_| {
    format!(
      "couldn't create storage directory '{}'",
      storage_dir.display()
    )
  })?;

  for plugin in &config.plugins {
    download_plugin(&storage_dir, plugin).with_context(|_| {
      format!("couldn't download plugin '{}'", plugin.name)
    })?;
  }

  Ok(())
}

fn download_plugin(storage_dir: &Path, plugin: &Plugin) -> Result<(), Error> {
  if plugin.from == PluginSource::Local {
    return Ok(());
  }

  let plugin_dir = storage_dir.join(plugin.id());

  if plugin_dir.is_dir() {
    return Ok(());
  }

  fs::create_dir_all(&plugin_dir).with_context(|_| {
    format!(
      "couldn't create plugin directory '{}'",
      plugin_dir.display()
    )
  })?;

  let download_result: Result<(), Error> = match plugin.from {
    PluginSource::Git => clone_git_repository(&plugin.name, &plugin_dir),
    PluginSource::Url => download_file(&plugin.name, &plugin_dir),
    _ => unreachable!(),
  };

  log!();

  if let Err(error) = download_result {
    fs::remove_dir_all(&plugin_dir).unwrap();
    Err(error)?;
  }

  Ok(())
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
