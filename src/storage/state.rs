use std::collections::HashSet;

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use failure::*;

use crate::config::{Plugin, PluginSource};

type StateData = HashSet<String>;

#[derive(Debug)]
pub struct State {
  path: PathBuf,
}

impl State {
  pub fn new(path: PathBuf) -> Self {
    Self { path }
  }

  pub fn is_plugin_downloaded(&self, plugin: &Plugin) -> Fallible<bool> {
    Ok(if plugin.from == PluginSource::Local {
      true
    } else {
      let downloaded_plugins = self.read()?;
      downloaded_plugins.contains(&plugin.id())
    })
  }

  pub fn add_downloaded_plugin(&self, plugin: &Plugin) -> Fallible<()> {
    let mut data = self.read()?;

    let changed = data.insert(plugin.id());
    if changed {
      self.write(&data)?;
    }

    Ok(())
  }

  fn read(&self) -> Fallible<StateData> {
    if !self.path.exists() {
      self.write(&HashSet::new())?;
    }

    let file = self.open()?;

    let reader = BufReader::new(&file);
    let data: StateData =
      bincode::deserialize_from(reader).with_context(|_| {
        format!("couldn't deserialize data from file '{}'", self.path.display())
      })?;

    Ok(data)
  }

  fn write(&self, data: &StateData) -> Fallible<()> {
    let file = self.open()?;

    let writer = BufWriter::new(file);
    bincode::serialize_into(writer, data).with_context(|_| {
      format!("couldn't serialize data into file '{}'", self.path.display())
    })?;

    Ok(())
  }

  fn open(&self) -> Fallible<File> {
    let file = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .open(&self.path)
      .with_context(|_| {
        format!("couldn't open file '{}'", self.path.display())
      })?;
    Ok(file)
  }
}
