use failure::*;
use std::fmt::Write;

use globset::{Glob, GlobSet, GlobSetBuilder};
use std::path::Path;
use walkdir::WalkDir;

use crate::config::Plugin;
use crate::storage::Storage;

pub fn generate(storage: &Storage, plugins: &[Plugin]) -> Fallible<String> {
  let mut script = String::new();

  macro_rules! write_script {
    ($($arg:tt)*) => {
      writeln!(script, $($arg)*).unwrap()
    };
  }

  macro_rules! write_block {
    ($name:expr, $body:block) => {
      write_script!("### {}", $name);
      $body
      write_script!("### end of {}", $name);
      write_script!();
    };
  }

  macro_rules! write_hook {
    ($name:expr, $body:expr) => {
      if !$body.is_empty() {
        write_block!($name, {
          write_script!("{}", $body);
        });
      }
    };
  }

  for plugin in plugins {
    write_script!("### plugin {:?} from {:?}", plugin.name, plugin.from);

    if !plugin.when.is_empty() {
      write_script!("if {}; then", plugin.when);
    }

    let plugin_dir = storage.plugin_dir(&plugin);
    write_script!("zshrug_plugin_dir={}", zsh_quote_path(&plugin_dir));
    write_script!();

    write_hook!("before_load", plugin.before_load);

    write_block!("load", {
      let ignored = compile_patterns(&plugin.ignore)?;
      for pattern in &plugin.load {
        let glob = compile_pattern(pattern)?.compile_matcher();

        for entry in WalkDir::new(&plugin_dir)
          .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        {
          let entry: walkdir::DirEntry = entry.with_context(|_| {
            format!(
              "couldn't get plugin directory contents: {}",
              plugin_dir.display()
            )
          })?;

          let full_path = entry.into_path();
          let short_path = full_path.strip_prefix(&plugin_dir).unwrap();

          if glob.is_match(&short_path) && !ignored.is_match(&short_path) {
            write_script!("source {}", zsh_quote_path(&full_path));
          }
        }
      }
    });

    write_hook!("after_load", plugin.after_load);

    if !plugin.when.is_empty() {
      write_script!("fi");
    }
  }

  write_script!("unset plugin_dir");

  Ok(script)
}

fn compile_patterns(patterns: &[String]) -> Fallible<GlobSet> {
  let mut builder = GlobSetBuilder::new();

  for pattern in patterns {
    builder.add(compile_pattern(pattern)?);
  }

  let glob_set = builder
    .build()
    .with_context(|_| format!("couldn't compile glob set: {:?}", patterns))?;

  Ok(glob_set)
}

fn compile_pattern(pattern: &str) -> Fallible<Glob> {
  let glob = Glob::new(&pattern)
    .with_context(|_| format!("couldn't compile glob: {}", pattern))?;
  Ok(glob)
}

fn zsh_quote_path(path: &Path) -> String {
  let path_str: String = path.display().to_string();
  format!("'{}'", path_str.replace('\'', "'\\''"))
}
