use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use config::*;

pub fn storage_directory() -> PathBuf {
  dirs::cache_dir().unwrap().join(env!("CARGO_PKG_NAME"))
}

pub fn download_plugins(config: &Config) -> io::Result<()> {
  let storage_dir = storage_directory();
  fs::create_dir_all(&storage_dir)?;

  for plugin in &config.plugins {
    if plugin.from == PluginSource::Local {
      continue;
    }

    let plugin_dir = storage_dir.join(plugin.id());

    if plugin_dir.is_dir() {
      continue;
    }

    fs::create_dir_all(&plugin_dir)?;

    let download_result: io::Result<()> = match plugin.from {
      PluginSource::Git => clone_git_repository(&plugin.name, &plugin_dir),
      PluginSource::Url => download_file(&plugin.name, &plugin_dir),
      _ => unreachable!(),
    };

    println!();

    if let Err(error) = download_result {
      fs::remove_dir_all(&plugin_dir).unwrap();
      return Err(error);
    }
  }

  Ok(())
}

fn clone_git_repository(repo: &str, dir: &Path) -> io::Result<()> {
  println!("Cloning git repository '{}'...", repo);

  let mut cmd = Command::new("git")
    .arg("clone")
    .arg(repo)
    .arg(dir)
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .spawn()?;

  let exit_code = cmd.wait()?;
  assert!(exit_code.success());

  Ok(())
}

fn download_file(url: &str, dir: &Path) -> io::Result<()> {
  println!("Downloading '{}'...", url);

  let mut cmd = Command::new("wget")
    .arg("-P")
    .arg(dir)
    .arg(url)
    .stdin(Stdio::inherit())
    .stdout(Stdio::inherit())
    .spawn()?;

  let exit_code = cmd.wait()?;
  assert!(exit_code.success());

  Ok(())
}
