use std::{
    collections::BTreeMap,
    fmt::Display,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Deserializer};

use crate::config::Name;

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct SourcePath(String);

impl Display for SourcePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
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

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(untagged)]
pub enum Value {
    InlinePath(PathBuf),
    CompositeValue {
        path: PathBuf,
        #[serde(default)]
        config: Config,
    },
}

impl Value {
    pub fn path(&self) -> &Path {
        match self {
            Value::InlinePath(path_buf) => path_buf.as_path(),
            Value::CompositeValue { path, .. } => path.as_path(),
        }
    }
}

#[derive(Debug, Default, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Config {
    #[serde(flatten)]
    encryption: Encryption,
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Module {
    pub name: Name,
    #[serde(flatten)]
    pub values: BTreeMap<SourcePath, Value>,
    #[serde(default)]
    pub config: Config,
}
