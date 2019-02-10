use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};

#[derive(Deserialize, Debug)]
pub struct Config {
  pub plugins: Vec<Plugin>,
}

#[derive(Debug, Eq, PartialEq, Deserialize)]
pub struct Plugin {
  #[serde(default)]
  pub when: String,

  pub name: String,

  #[serde(default)]
  pub from: PluginSource,

  #[serde(default)]
  pub build: String,

  #[serde(default)]
  pub before_load: String,

  #[serde(default)]
  pub after_load: String,

  #[serde(deserialize_with = "deserialize_patterns")]
  #[serde(default = "default_load_patterns")]
  pub load: Vec<String>,

  #[serde(deserialize_with = "deserialize_patterns")]
  #[serde(default)]
  pub ignore: Vec<String>,

  #[serde(default)]
  pub path: Vec<PathArrayChange>,

  #[serde(default)]
  pub fpath: Vec<PathArrayChange>,

  #[serde(default)]
  pub manpath: Vec<PathArrayChange>,
}

impl Plugin {
  pub fn id(&self) -> String {
    let hashed_str = format!("{:?}:{}:{}", self.from, self.name, self.build);
    format!("{:x}", md5::compute(hashed_str))
  }
}

fn default_load_patterns() -> Vec<String> {
  vec![".".to_string()]
}

fn deserialize_patterns<'de, D>(
  deserializer: D,
) -> Result<Vec<String>, D::Error>
where
  D: Deserializer<'de>,
{
  struct PatternsVisitor;

  impl<'de> Visitor<'de> for PatternsVisitor {
    type Value = Vec<String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
      write!(formatter, "string or sequence")
    }

    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
    where
      E: de::Error,
    {
      Ok(vec![value.to_string()])
    }

    fn visit_seq<A>(self, seq: A) -> Result<Self::Value, A::Error>
    where
      A: de::SeqAccess<'de>,
    {
      Deserialize::deserialize(de::value::SeqAccessDeserializer::new(seq))
    }
  }

  deserializer.deserialize_any(PatternsVisitor)
}

#[derive(Debug, Eq, PartialEq, Deserialize)]
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

#[derive(Debug, Eq, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum PathArrayChange {
  Append(String),
  Prepend(String),
}
