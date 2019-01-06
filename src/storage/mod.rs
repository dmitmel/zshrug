use std::borrow::Cow;

use cluFlock::{ExclusiveSliceLock, Flock};
use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use failure::*;

use crate::config::*;

mod state;
use self::state::{PluginState, State};

#[derive(Debug)]
pub struct Storage {
  root: PathBuf,
  state: State,
}

impl Storage {
  pub fn init(root: &Path) -> Fallible<Self> {
    fs::create_dir_all(root).with_context(|_| {
      format!("couldn't create storage directory '{}'", root.display())
    })?;

    let state_path = root.join("state");
    let state = State::new(state_path);

    Ok(Self { root: root.to_path_buf(), state })
  }

  pub fn plugin_dir<'p>(&self, plugin: &'p Plugin) -> Cow<'p, Path> {
    if plugin.from == PluginSource::Local {
      Cow::Borrowed(Path::new(&plugin.name))
    } else {
      Cow::Owned(self.root.join("plugins").join(plugin.id()))
    }
  }

  pub fn ensure_plugins_are_installed<'p>(
    &mut self,
    plugins: &'p [&'p Plugin],
  ) -> Fallible<Vec<&'p Plugin>> {
    let mut plugins_to_install: Vec<&Plugin> = vec![];
    let mut installed_plugins: Vec<&Plugin> = plugins.to_vec();

    for plugin in plugins {
      if self.get_plugin_state(plugin)? != Some(PluginState::Built) {
        plugins_to_install.push(plugin);
      }
    }

    if !plugins_to_install.is_empty() {
      let lock_path = self.root.join("lock");
      let lock_file = File::create(&lock_path)?;
      let _lock = exclusively_lock_file(&lock_file).with_context(|_| {
        format!("couldn't lock file '{}'", lock_path.display())
      })?;

      for plugin in plugins_to_install {
        let installation_result =
          self.install_plugin(plugin).with_context(|_| {
            format!(
              "couldn't install plugin {:?} from {:?}",
              plugin.name, plugin.from
            )
          });

        if let Err(plugin_error) = installation_result {
          installed_plugins.remove(
            installed_plugins.iter().position(|x| *x == plugin).unwrap(),
          );
          crate::log::log_error(&plugin_error);
        }
      }
    }

    Ok(installed_plugins)
  }

  fn install_plugin(&mut self, plugin: &Plugin) -> Fallible<()> {
    let plugin_dir = self.plugin_dir(&plugin);

    if self.get_plugin_state(plugin)? == None {
      download_plugin(&plugin, &plugin_dir)
        .context("couldn't download plugin")?;
      self.set_plugin_state(&plugin, PluginState::Downloaded)?;
    }

    if self.get_plugin_state(plugin)? == Some(PluginState::Downloaded) {
      build_plugin(&plugin, &plugin_dir).context("couldn't build plugin")?;
      self.set_plugin_state(&plugin, PluginState::Built)?;
    }

    Ok(())
  }

  fn get_plugin_state(&self, plugin: &Plugin) -> Fallible<Option<PluginState>> {
    let result = self
      .state
      .get_plugin_state(plugin)
      .context("couldn't read storage state")?;
    Ok(result)
  }

  fn set_plugin_state(
    &self,
    plugin: &Plugin,
    state: PluginState,
  ) -> Fallible<()> {
    self
      .state
      .set_plugin_state(plugin, state)
      .context("couldn't read storage state")?;
    Ok(())
  }
}

fn exclusively_lock_file(file: &File) -> io::Result<ExclusiveSliceLock> {
  file.try_exclusive_lock().and_then(|result| match result {
    Some(lock) => Ok(lock),
    None => {
      info!("waiting for another process to unlock the lock file");
      file.exclusive_lock()
    }
  })
}

fn download_plugin(plugin: &Plugin, directory: &Path) -> Fallible<()> {
  info!("downloading plugin {:?} from {:?}", plugin.name, plugin.from);

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

fn build_plugin(plugin: &Plugin, directory: &Path) -> Fallible<()> {
  if plugin.build.is_empty() {
    return Ok(());
  }

  info!("running build command: {}", plugin.build);

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
  info!("cloning git repository '{}'...", repo);

  let exit_status = Command::new("git")
    .arg("clone")
    .arg(repo)
    .arg(directory)
    .stdout(Stdio::null())
    .status()
    .context("couldn't run git")?;

  ensure!(exit_status.success(), "git has exited with an error");
  Ok(())
}

fn download_file(url: &str, directory: &Path) -> Fallible<()> {
  info!("downloading '{}'...", url);

  let exit_status = Command::new("wget")
    .arg("--directory-prefix")
    .arg(directory)
    .arg(url)
    .stdout(Stdio::null())
    .status()
    .context("couldn't run wget")?;

  ensure!(exit_status.success(), "wget has exited with an error");
  Ok(())
}
