use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use std::fs::{File, OpenOptions};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;

use failure::*;

use crate::config::{Plugin, PluginSource};

type StateData = HashMap<String, PluginState>;

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub enum PluginState {
  NotDownloaded,
  Downloaded,
  Built,
}

#[derive(Debug)]
pub struct StateFile {
  path: PathBuf,
  data: StateData,
}

impl StateFile {
  pub fn init(path: PathBuf) -> Fallible<Self> {
    let mut state = Self { path, data: StateData::default() };

    if !state.path.exists() {
      state.write()?;
    } else {
      state.read()?;
    }

    Ok(state)
  }

  pub fn get_plugin_state(&self, plugin: &Plugin) -> Fallible<PluginState> {
    Ok(if plugin.from == PluginSource::Local {
      PluginState::Downloaded
    } else {
      self.data.get(&plugin.id()).cloned().unwrap_or(PluginState::NotDownloaded)
    })
  }

  pub fn set_plugin_state(
    &mut self,
    plugin: &Plugin,
    state: PluginState,
  ) -> Fallible<()> {
    self.data.insert(plugin.id(), state);
    self.write()?;

    Ok(())
  }

  pub fn read(&mut self) -> Fallible<()> {
    let file = self.open()?;

    let reader = BufReader::new(file);
    self.data = serde_yaml::from_reader(reader).with_context(|_| {
      format!("couldn't deserialize data from file '{}'", self.path.display())
    })?;

    Ok(())
  }

  pub fn write(&mut self) -> Fallible<()> {
    let file = self.open()?;
    file.set_len(0).unwrap();

    let writer = BufWriter::new(file);
    serde_yaml::to_writer(writer, &self.data).with_context(|_| {
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
