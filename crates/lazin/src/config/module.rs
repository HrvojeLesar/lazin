use std::{collections::BTreeMap, fmt::Display, path::PathBuf};

use serde::{Deserialize, Deserializer};

use crate::config::Name;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourcePath(String);

impl Display for SourcePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl AsRef<str> for SourcePath {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl From<SourcePath> for PathBuf {
    fn from(value: SourcePath) -> Self {
        PathBuf::from(value.0)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encryption {
    #[default]
    Disabled,
    Enabled {
        recipient: String,
    },
}

impl<'de> Deserialize<'de> for Encryption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct EncryptionRaw {
            encrypt: bool,
            recipient: Option<String>,
        }

        let raw = EncryptionRaw::deserialize(deserializer)?;
        match (raw.encrypt, raw.recipient) {
            (false, _) => Ok(Encryption::Disabled),
            (true, Some(recipient)) => Ok(Encryption::Enabled { recipient }),
            // TODO: change this to a module level recipient or a configured
            // recipient variable, if none of those are set error.
            // DeserializeSeed can be used to provide values
            (true, None) => Err(serde::de::Error::custom(
                "encrypt = true requires a `recipient` field",
            )),
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Value {
    pub path: PathBuf,
    pub config: Config,
}

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        pub enum ValueRaw {
            InlinePath(PathBuf),
            CompositeValue {
                path: PathBuf,
                #[serde(default)]
                config: Config,
            },
        }

        let raw = ValueRaw::deserialize(deserializer)?;
        Ok(match raw {
            ValueRaw::InlinePath(path) => Self {
                path,
                config: Config::default(),
            },
            ValueRaw::CompositeValue { path, config } => Self { path, config },
        })
    }
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Config {
    #[serde(flatten)]
    pub encryption: Encryption,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Module {
    pub name: Name,
    #[serde(flatten)]
    pub values: BTreeMap<SourcePath, Value>,
    #[serde(default)]
    pub config: Config,
}
