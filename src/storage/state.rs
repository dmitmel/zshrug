use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use failure::*;

use crate::config::{Plugin, PluginSource};

type StateData = HashMap<String, PluginState>;

#[derive(Debug, Eq, PartialEq, Serialize, Deserialize)]
pub enum PluginState {
  NotDownloaded,
  Downloaded,
  Built,
}

#[derive(Debug)]
pub struct StateFile {
  path: PathBuf,
}

impl StateFile {
  pub fn new(path: PathBuf) -> Self {
    Self { path }
  }

  pub fn get_plugin_state(&self, plugin: &Plugin) -> Fallible<PluginState> {
    Ok(if plugin.from == PluginSource::Local {
      PluginState::Downloaded
    } else {
      let mut data = self.read()?;
      data.remove(&plugin.id()).unwrap_or(PluginState::NotDownloaded)
    })
  }

  pub fn set_plugin_state(
    &self,
    plugin: &Plugin,
    state: PluginState,
  ) -> Fallible<()> {
    let mut data = self.read()?;
    data.insert(plugin.id(), state);
    self.write(&data)?;

    Ok(())
  }

  fn read(&self) -> Fallible<StateData> {
    if !self.path.exists() {
      self.write(&StateData::default())?;
    }

    let file = self.open()?;

    let reader = BufReader::new(file);
    let data: StateData =
      serde_yaml::from_reader(reader).with_context(|_| {
        format!("couldn't deserialize data from file '{}'", self.path.display())
      })?;

    Ok(data)
  }

  fn write(&self, data: &StateData) -> Fallible<()> {
    let file = self.open()?;
    file.set_len(0).unwrap();

    let writer = BufWriter::new(file);
    serde_yaml::to_writer(writer, data).with_context(|_| {
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
