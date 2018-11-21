extern crate dirs;
extern crate md5;

extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

mod config;
mod storage;

fn main() {
  use std::io::{self, Read};

  let input: String = {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    buffer
  };

  let config: config::Config = serde_yaml::from_str(&input).unwrap();
  storage::download_plugins(&config).unwrap();
}
