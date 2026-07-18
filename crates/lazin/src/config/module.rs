use std::{collections::BTreeMap, fmt::Display, path::PathBuf};

use serde::{
    Deserialize, Deserializer,
    de::{self, DeserializeSeed},
};

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

impl Encryption {
    fn recipient(&self) -> Option<&str> {
        match self {
            Encryption::Disabled => None,
            Encryption::Enabled { recipient } => Some(recipient),
        }
    }
}

impl<'de> Deserialize<'de> for Encryption {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        EncryptionSeed {
            fallback_recipient: None,
        }
        .deserialize(deserializer)
    }
}

struct EncryptionSeed<'a> {
    fallback_recipient: Option<&'a str>,
}

impl<'de, 'a> DeserializeSeed<'de> for EncryptionSeed<'a> {
    type Value = Encryption;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
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
            (true, None) => match self.fallback_recipient {
                Some(recipient) => Ok(Encryption::Enabled {
                    recipient: recipient.to_string(),
                }),
                None => Err(de::Error::custom(
                    "encrypt = true requires a `recipient` field (none set here or on the module)",
                )),
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Value {
    pub path: PathBuf,
    pub config: Config,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Config {
    pub encryption: Encryption,
}

impl<'de> Deserialize<'de> for Config {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        ConfigSeed {
            fallback_recipient: None,
        }
        .deserialize(deserializer)
    }
}

struct ConfigSeed<'a> {
    fallback_recipient: Option<&'a str>,
}

impl<'de, 'a> DeserializeSeed<'de> for ConfigSeed<'a> {
    type Value = Config;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let encryption = EncryptionSeed {
            fallback_recipient: self.fallback_recipient,
        }
        .deserialize(deserializer)?;
        Ok(Config { encryption })
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Module {
    pub name: Name,
    pub values: BTreeMap<SourcePath, Value>,
    pub config: Config,
}

impl<'de> Deserialize<'de> for Module {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let mut value = toml::Value::deserialize(deserializer)?;
        let table = value
            .as_table_mut()
            .ok_or_else(|| de::Error::custom("expected `Module` to be a table"))?;

        let name = table
            .remove("name")
            .ok_or_else(|| de::Error::custom("missing field `name`"))?;
        let name = Name::deserialize(name).map_err(de::Error::custom)?;

        let config = match table.remove("config") {
            Some(v) => ConfigSeed {
                fallback_recipient: None,
            }
            .deserialize(v)
            .map_err(de::Error::custom)?,
            None => Config::default(),
        };

        let module_recipient = config.encryption.recipient();

        let mut values = BTreeMap::new();
        for (key, raw_value) in table.iter() {
            let source_path = SourcePath::deserialize(toml::Value::String(key.clone()))
                .map_err(de::Error::custom)?;
            let value = ValueSeed {
                default_config: &config,
                fallback_recipient: module_recipient,
            }
            .deserialize(raw_value.clone())
            .map_err(de::Error::custom)?;
            values.insert(source_path, value);
        }

        Ok(Module {
            name,
            values,
            config,
        })
    }
}

struct ValueSeed<'a> {
    default_config: &'a Config,
    fallback_recipient: Option<&'a str>,
}

impl<'de, 'a> DeserializeSeed<'de> for ValueSeed<'a> {
    type Value = Value;

    fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer<'de>,
    {
        let raw = toml::Value::deserialize(deserializer)?;

        match raw {
            toml::Value::String(s) => Ok(Value {
                path: PathBuf::from(s),
                config: self.default_config.clone(),
            }),
            toml::Value::Table(mut table) => {
                let path = table
                    .remove("path")
                    .ok_or_else(|| de::Error::custom("missing field `path`"))?;
                let path = PathBuf::deserialize(path).map_err(de::Error::custom)?;

                let config = match table.remove("config") {
                    Some(config_value) => ConfigSeed {
                        fallback_recipient: self.fallback_recipient,
                    }
                    .deserialize(config_value)
                    .map_err(de::Error::custom)?,
                    None => self.default_config.clone(),
                };

                Ok(Value { path, config })
            }
            _ => Err(de::Error::custom(
                "expected a path string or a table with a `path` field",
            )),
        }
    }
}
