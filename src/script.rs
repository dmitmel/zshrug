use failure::*;
use std::fmt::Write;

use std::path::Path;

use crate::config::{PathArrayChange, Plugin};
use crate::storage::Storage;

const PLUGIN_PATH_VAR_NAME: &str = "zshrug_plugin_path";
const LOAD_PATTERNS_VAR_NAME: &str = "zshrug_load_patterns";
const IGNORE_PATTERNS_VAR_NAME: &str = "zshrug_ignore_patterns";
const SCRIPT_PATH_VAR_NAME: &str = "zshrug_script_path";
const IGNORE_PATTERN_VAR_NAME: &str = "zshrug_ignore_pattern";

pub fn generate(storage: &Storage, plugins: &[&Plugin]) -> Fallible<String> {
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

    use std::borrow::Cow;
    let plugin_dir: Cow<Path> = storage.plugin_dir(&plugin);
    write_script!("{}={}", PLUGIN_PATH_VAR_NAME, zsh_quote_path(&plugin_dir));
    write_script!();

    write_hook!("before_load", plugin.before_load);

    if !plugin.load.is_empty() {
      write_block!("load", {
        macro_rules! write_array {
          ($var:expr, $val:expr) => {
            write_script!("{}=(", $var);
            for item in &$val {
              write_script!("{}", zsh_quote_str(item));
            }
            write_script!(")");
          };
        }

        write_array!(LOAD_PATTERNS_VAR_NAME, plugin.load);
        write_array!(IGNORE_PATTERNS_VAR_NAME, plugin.ignore);

        write_script!(
          "for {script_path_var} in \"${plugin_path_var}/\"${{^~{load_patterns_var}}}(N); do",
          script_path_var = SCRIPT_PATH_VAR_NAME,
          plugin_path_var = PLUGIN_PATH_VAR_NAME,
          load_patterns_var = LOAD_PATTERNS_VAR_NAME,
        );

        if !plugin.ignore.is_empty() {
          write_script!(
            "for {ignore_pattern_var} in ${plugin_ignore_patterns}; do",
            ignore_pattern_var = IGNORE_PATTERN_VAR_NAME,
            plugin_ignore_patterns = IGNORE_PATTERNS_VAR_NAME,
          );
          write_script!(
            "[[ \"${script_path_var}\" == \"${plugin_path_var}/\"${{~{ignore_pattern_var}}} ]] && continue 2",
            script_path_var = SCRIPT_PATH_VAR_NAME,
            plugin_path_var = PLUGIN_PATH_VAR_NAME,
            ignore_pattern_var = IGNORE_PATTERN_VAR_NAME,
          );
          write_script!("done");
          write_script!("unset {}", IGNORE_PATTERN_VAR_NAME);
        }

        write_script!("source \"${}\"", SCRIPT_PATH_VAR_NAME);
        write_script!("done");
        write_script!("unset {}", SCRIPT_PATH_VAR_NAME);
      });
    }

    macro_rules! write_array_changes {
      ($var:expr, $changes:expr) => {
        if !$changes.is_empty() {
          write_block!($var, {
            for change in &$changes {
              use self::PathArrayChange::*;
              match change {
                Append(value) => {
                  write_script!(
                    "{array_var}=(${array_var} \"${plugin_path_var}/\"{val})",
                    plugin_path_var = PLUGIN_PATH_VAR_NAME,
                    array_var = $var,
                    val = zsh_quote_str(value),
                  );
                }
                Prepend(value) => {
                  write_script!(
                    "{array_var}=(\"${plugin_path_var}/\"{val} ${array_var})",
                    plugin_path_var = PLUGIN_PATH_VAR_NAME,
                    array_var = $var,
                    val = zsh_quote_str(value),
                  );
                }
              }
            }
          });
        }
      };
    }

    write_array_changes!("path", plugin.path);
    write_array_changes!("fpath", plugin.fpath);
    write_array_changes!("manpath", plugin.manpath);

    write_hook!("after_load", plugin.after_load);

    if !plugin.when.is_empty() {
      write_script!("fi");
    }
  }

  write_script!("unset {}", PLUGIN_PATH_VAR_NAME);
  write_script!("unset {}", LOAD_PATTERNS_VAR_NAME);
  write_script!("unset {}", IGNORE_PATTERNS_VAR_NAME);

  Ok(script)
}

fn zsh_quote_path(path: &Path) -> String {
  let path_str: String = path.display().to_string();
  zsh_quote_str(&path_str)
}

fn zsh_quote_str(s: &str) -> String {
  format!("'{}'", s.replace('\'', "'\\''"))
}
