use std::fmt::Display;

use crate::common::Key;

pub type DuplicateKeys = Vec<Key>;

pub struct DuplicateKeysError {
    pub duplicates: DuplicateKeys,
}

#[derive(Debug)]
pub enum Error {
    TomlParse(toml::de::Error),
    Io(std::io::Error),
    DuplicateKeys(DuplicateKeys),
    Custom(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TomlParse(error) => error.fmt(f),
            Error::Io(error) => error.fmt(f),
            Error::DuplicateKeys(error) => {
                write!(f, "{}", duplicate_keys_string(error))
            }
            Error::Custom(c) => write!(f, "{}", c),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Self::TomlParse(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<DuplicateKeysError> for Error {
    fn from(value: DuplicateKeysError) -> Self {
        Self::DuplicateKeys(value.duplicates)
    }
}

fn duplicate_keys_string(duplicates: &[Key]) -> String {
    duplicates
        .iter()
        .map(|dup| dup.str())
        .collect::<Vec<_>>()
        .join(", ")
}
