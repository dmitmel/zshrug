const INIT_COMMAND: &str = "init";
const LIST_COMMAND: &str = "list";
const UPGRADE_COMMAND: &str = "upgrade";
const STORAGE_COMMAND: &str = "storage";
const CLEANUP_COMMAND: &str = "cleanup";
const COMPLETION_COMMAND: &str = "completion";

pub struct Options {
  pub command: Command,
}

pub enum Command {
  Init,
  List,
  Upgrade,
  Storage,
  Cleanup,
}

pub fn parse_options() -> Options {
  let mut parser = create_parser();
  let matches = parser.clone().get_matches();

  let command = match matches.subcommand() {
    (INIT_COMMAND, Some(_init_matches)) => Command::Init,
    (LIST_COMMAND, Some(_list_matches)) => Command::List,
    (UPGRADE_COMMAND, Some(_upgrade_matches)) => Command::Upgrade,
    (STORAGE_COMMAND, Some(_storage_matches)) => Command::Storage,
    (CLEANUP_COMMAND, Some(_cleanup_matches)) => Command::Cleanup,
    (COMPLETION_COMMAND, Some(_completion_matches)) => {
      let name = parser.get_name().to_string();
      parser.gen_completions_to(name, clap::Shell::Zsh, &mut std::io::stdout());
      std::process::exit(0);
    }
    _ => unreachable!(),
  };

  Options { command }
}

fn create_parser() -> clap::App<'static, 'static> {
  use clap::*;

  app_from_crate!()
    .setting(AppSettings::SubcommandRequiredElseHelp)
    .setting(AppSettings::GlobalVersion)
    .setting(AppSettings::ColoredHelp)
    .subcommand(
      SubCommand::with_name(INIT_COMMAND)
        .about("generates initialization script"),
    )
    .subcommand(
      SubCommand::with_name(LIST_COMMAND).about("lists active plugins"),
    )
    .subcommand(
      SubCommand::with_name(UPGRADE_COMMAND).about("upgrades active plugins"),
    )
    .subcommand(
      SubCommand::with_name(STORAGE_COMMAND)
        .about("prints path to the storage directory"),
    )
    .subcommand(
      SubCommand::with_name(CLEANUP_COMMAND).about("deletes active plugins"),
    )
    .subcommand(
      SubCommand::with_name(COMPLETION_COMMAND)
        .about("generates completion script"),
    )
}
