use failure::*;
use std::fmt::Write;

use std::borrow::Cow;
use std::path::Path;

use crate::config::{PathArrayChange, Plugin};
use crate::storage::Storage;

const PLUGIN_PATH_VAR_NAME: &str = "zshrug_plugin_path";
const LOAD_PATTERNS_VAR_NAME: &str = "zshrug_load_patterns";
const IGNORE_PATTERNS_VAR_NAME: &str = "zshrug_ignore_patterns";
const SOURCE_FUNC_NAME: &str = "zshrug_source";
const LOAD_PLUGIN_FUNC_NAME: &str = "zshrug_load_plugin";

macro_rules! wln {
  ($($arg:tt)*) => {
    writeln!($($arg)*).unwrap()
  };
}

pub fn generate(storage: &Storage, plugins: &[&Plugin]) -> Fallible<String> {
  let mut script = String::new();

  write_helper_functions(&mut script);

  for plugin in plugins {
    write_plugin(&mut script, plugin, storage.plugin_dir(&plugin));
  }

  write_cleanup_section(&mut script);

  Ok(script)
}

fn write_helper_functions(script: &mut String) {
  wln!(
    script,
    r#"
{source_func_name}() {{
  local file="$1"; shift
  [[ -d "$file" ]] && file=$(echo "$file"/*.(plugin.zsh|zsh-theme)(N[1]))
  [[ -n "$file" ]] && source "$file" "$@"
}}

{load_plugin_func_name}() {{
  local script_path ignore_pattern
  for script_path in "$zshrug_plugin_path/"${{^~zshrug_load_patterns}}(N); do
    for ignore_pattern in $zshrug_ignore_patterns; do
      [[ "$script_path" == "$zshrug_plugin_path/"${{~ignore_pattern}} ]] && continue 2
    done
    zshrug_source "$script_path"
  done
}}
"#,
    source_func_name = SOURCE_FUNC_NAME,
    load_plugin_func_name = LOAD_PLUGIN_FUNC_NAME,
  );
}

fn write_cleanup_section(script: &mut String) {
  wln!(script, "unset {}", PLUGIN_PATH_VAR_NAME);
  wln!(script, "unset {}", LOAD_PATTERNS_VAR_NAME);
  wln!(script, "unset {}", IGNORE_PATTERNS_VAR_NAME);
}

fn write_plugin(script: &mut String, plugin: &Plugin, plugin_dir: Cow<Path>) {
  wln!(script, "### plugin {:?} from {:?}", plugin.name, plugin.from);

  if !plugin.when.is_empty() {
    wln!(script, "if {}; then", plugin.when);
  }

  wln!(
    script,
    "{path_var_name}={plugin_dir}",
    path_var_name = PLUGIN_PATH_VAR_NAME,
    plugin_dir = quote_for_shell(&plugin_dir.display().to_string()),
  );
  wln!(script);

  write_block(script, "before_load", &plugin.before_load);

  if !plugin.load.is_empty() {
    write_array(script, LOAD_PATTERNS_VAR_NAME, &plugin.load);
    write_array(script, IGNORE_PATTERNS_VAR_NAME, &plugin.ignore);
    wln!(script, "{}", LOAD_PLUGIN_FUNC_NAME);
    wln!(script);
  }

  write_array_changes(script, "path", &plugin.path);
  write_array_changes(script, "fpath", &plugin.fpath);
  write_array_changes(script, "manpath", &plugin.manpath);

  write_block(script, "after_load", &plugin.after_load);

  if !plugin.when.is_empty() {
    wln!(script, "fi");
  }

  wln!(script);
}

fn write_block(script: &mut String, name: &str, body: &str) {
  if body.is_empty() {
    return;
  }

  wln!(script, "## {}", name);
  wln!(script, "{}", body);
}

fn write_array(script: &mut String, variable_name: &str, values: &[String]) {
  wln!(script, "{}=(", variable_name);
  for value in values {
    wln!(script, "{}", quote_for_shell(value));
  }
  wln!(script, ")");
}

fn write_array_changes(
  script: &mut String,
  var: &str,
  changes: &[PathArrayChange],
) {
  if changes.is_empty() {
    return;
  }

  for change in changes {
    use self::PathArrayChange::*;
    match change {
      Append(value) => {
        wln!(
          script,
          "{array_var}=(${array_var} \"${plugin_path_var}/\"{val})",
          plugin_path_var = PLUGIN_PATH_VAR_NAME,
          array_var = var,
          val = quote_for_shell(value),
        );
      }
      Prepend(value) => {
        wln!(
          script,
          "{array_var}=(\"${plugin_path_var}/\"{val} ${array_var})",
          plugin_path_var = PLUGIN_PATH_VAR_NAME,
          array_var = var,
          val = quote_for_shell(value),
        );
      }
    }
  }
}

fn quote_for_shell(s: &str) -> String {
  format!("'{}'", s.replace('\'', "'\\''"))
}
