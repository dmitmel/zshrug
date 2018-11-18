extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_yaml;

mod config;
use config::*;

fn main() {
  use std::io::{self, Read};

  let input: String = {
    let mut buffer = String::new();
    io::stdin().read_to_string(&mut buffer).unwrap();
    buffer
  };

  let config: Config = serde_yaml::from_str(&input).unwrap();
  println!("{:#?}", config);
}
