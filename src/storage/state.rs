use std::collections::HashSet;

use std::fs::{File, OpenOptions};
use std::io::{self, BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;

use failure::*;

use crate::config::Plugin;

#[derive(Debug)]
pub struct State {
  path: PathBuf,
  file: File,
  downloaded_plugins: HashSet<String>,
}

impl State {
  pub fn load(path: PathBuf) -> Fallible<Self> {
    let file: File = OpenOptions::new()
      .read(true)
      .write(true)
      .create(true)
      .open(&path)
      .with_context(|_| format!("couldn't open file '{}'", path.display()))?;

    let downloaded_plugins = {
      let reader = BufReader::new(&file);
      reader
        .lines()
        .collect::<io::Result<HashSet<String>>>()
        .with_context(|_| format!("couldn't read file '{}'", path.display()))?
    };

    Ok(Self {
      path,
      file,
      downloaded_plugins,
    })
  }

  pub fn is_plugin_downloaded(&self, plugin: &Plugin) -> bool {
    self.downloaded_plugins.contains(&plugin.id())
  }

  pub fn add_downloaded_plugin(&mut self, plugin: &Plugin) -> Fallible<()> {
    let changed = self.downloaded_plugins.insert(plugin.id());
    if changed {
      self.save().context("couldn't save storage state")?
    }
    Ok(())
  }

  fn save(&mut self) -> Fallible<()> {
    self.file.set_len(0).unwrap();
    self.file.seek(SeekFrom::Start(0)).unwrap();

    let mut buf_writer = BufWriter::new(&self.file);
    for id in &self.downloaded_plugins {
      writeln!(buf_writer, "{}", id).with_context(|_| {
        format!("couldn't write to file '{}'", self.path.display())
      })?;
    }

    buf_writer.flush().unwrap();

    Ok(())
  }
}
