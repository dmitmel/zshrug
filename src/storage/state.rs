use std::collections::HashSet;

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::PathBuf;

use failure::*;

use crate::config::{Plugin, PluginSource};

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

  fn read(&self) -> Fallible<HashSet<String>> {
    let file = self.open()?;

    let reader = BufReader::new(&file);
    let downloaded_plugins =
      reader.lines().collect::<io::Result<HashSet<String>>>().with_context(
        |_| format!("couldn't read file '{}'", self.path.display()),
      )?;

    Ok(downloaded_plugins)
  }

  pub fn add_downloaded_plugin(&self, plugin: &Plugin) -> Fallible<()> {
    let mut downloaded_plugins = self.read()?;

    let changed = downloaded_plugins.insert(plugin.id());
    if changed {
      let file = self.open()?;

      let mut buf_writer = BufWriter::new(file);
      for id in &downloaded_plugins {
        writeln!(buf_writer, "{}", id).with_context(|_| {
          format!("couldn't write to file '{}'", self.path.display())
        })?;
      }

      buf_writer.flush().unwrap();
    }

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
