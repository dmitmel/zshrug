use std::fmt;

use serde::de::{self, Visitor};
use serde::{Deserialize, Deserializer};

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

  #[serde(deserialize_with = "deserialize_patterns")]
  #[serde(default)]
  pub load: Vec<String>,

  #[serde(deserialize_with = "deserialize_patterns")]
  #[serde(default)]
  pub ignore: Vec<String>,
}

impl Plugin {
  pub fn id(&self) -> String {
    let id_str = format!("{:?}:{}", self.from, self.name);
    format!("{:x}", md5::compute(id_str))
  }
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
