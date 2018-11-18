use std::fmt;

use serde::de::{self, Deserialize, Deserializer, SeqAccess, Visitor};

#[derive(Deserialize, Debug)]
pub struct Config {
  pub plugins: Vec<Plugin>,
}

#[derive(Deserialize, Debug)]
pub struct Plugin {
  #[serde(default)]
  pub when: String,

  pub name: String,

  #[serde(default)]
  pub from: PluginSource,

  #[serde(default)]
  pub on_build: String,

  #[serde(default)]
  pub on_setup: String,

  #[serde(default)]
  pub on_load: String,

  #[serde(default)]
  pub load: Patterns,

  #[serde(default)]
  pub ignore: Patterns,
}

#[derive(Debug, Default)]
pub struct Patterns(pub Vec<String>);

impl<'de> Deserialize<'de> for Patterns {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct PatternsVisitor;

    impl<'de> Visitor<'de> for PatternsVisitor {
      type Value = Patterns;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(formatter, "string or sequence")
      }

      fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
      where
        E: de::Error,
      {
        Ok(Patterns(vec![value.to_string()]))
      }

      fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
      where
        A: SeqAccess<'de>,
      {
        Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))
          .map(Patterns)
      }
    }

    deserializer.deserialize_any(PatternsVisitor)
  }
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "kebab-case")]
pub enum PluginSource {
  Git,
  Url,
  Local,
}

impl Default for PluginSource {
  fn default() -> Self {
    PluginSource::Git
  }
}
