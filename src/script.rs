use std::borrow::Cow;
use std::path::Path;

use crate::config::{PathArrayChange, Plugin};
use crate::storage::Storage;

const PLUGIN_PATH_VAR_NAME: &str = "zshrug_plugin_path";
const LOAD_PATTERNS_VAR_NAME: &str = "zshrug_load_patterns";
const IGNORE_PATTERNS_VAR_NAME: &str = "zshrug_ignore_patterns";
const SOURCE_FUNC_NAME: &str = "zshrug_source";
const LOAD_PLUGIN_FUNC_NAME: &str = "zshrug_load_plugin";

pub fn generate(storage: &Storage, plugins: &[&Plugin]) {
  write_helper_functions();

  for plugin in plugins {
    write_plugin(plugin, storage.plugin_dir(&plugin));
  }

  write_cleanup_section();
}

fn write_helper_functions() {
  println!(
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

fn write_cleanup_section() {
  println!("unset {}", PLUGIN_PATH_VAR_NAME);
  println!("unset {}", LOAD_PATTERNS_VAR_NAME);
  println!("unset {}", IGNORE_PATTERNS_VAR_NAME);
}

fn write_plugin(plugin: &Plugin, plugin_dir: Cow<Path>) {
  println!("### plugin {:?} from {:?}", plugin.name, plugin.from);

  if !plugin.when.is_empty() {
    println!("if {}; then", plugin.when);
  }

  println!(
    "{path_var_name}={plugin_dir}",
    path_var_name = PLUGIN_PATH_VAR_NAME,
    plugin_dir = quote_for_shell(&plugin_dir.display().to_string()),
  );
  println!();

  write_block("before_load", &plugin.before_load);

  if !plugin.load.is_empty() {
    write_array(LOAD_PATTERNS_VAR_NAME, &plugin.load);
    write_array(IGNORE_PATTERNS_VAR_NAME, &plugin.ignore);
    println!("{}", LOAD_PLUGIN_FUNC_NAME);
    println!();
  }

  write_array_changes("path", &plugin.path);
  write_array_changes("fpath", &plugin.fpath);
  write_array_changes("manpath", &plugin.manpath);

  write_block("after_load", &plugin.after_load);

  if !plugin.when.is_empty() {
    println!("fi");
  }

  println!();
}

fn write_block(name: &str, body: &str) {
  if body.is_empty() {
    return;
  }

  println!("## {}", name);
  println!("{}", body);
}

fn write_array(variable_name: &str, values: &[String]) {
  println!("{}=(", variable_name);
  for value in values {
    println!("{}", quote_for_shell(value));
  }
  println!(")");
}

fn write_array_changes(var: &str, changes: &[PathArrayChange]) {
  if changes.is_empty() {
    return;
  }

  for change in changes {
    use self::PathArrayChange::*;
    match change {
      Append(value) => {
        println!(
          "{array_var}=(${array_var} \"${plugin_path_var}/\"{val})",
          plugin_path_var = PLUGIN_PATH_VAR_NAME,
          array_var = var,
          val = quote_for_shell(value),
        );
      }
      Prepend(value) => {
        println!(
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
