use std::{fmt::Display, path::Path};

use crate::common::Key;

pub type DuplicateKeys = Vec<Key>;

pub struct DuplicateKeysError {
    pub duplicates: DuplicateKeys,
}

pub type LazinResult<T> = Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    TomlParse(toml::de::Error),
    Io(std::io::Error),
    DuplicateKeys(DuplicateKeys),
    DirectoryDoesNotExist(String),
    Custom(&'static str),
}

impl Error {
    pub fn directory_does_not_exist(path: &Path) -> Self {
        Self::DirectoryDoesNotExist(format!("{}", path.display()))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TomlParse(error) => error.fmt(f),
            Error::Io(error) => error.fmt(f),
            Error::DuplicateKeys(error) => {
                write!(
                    f,
                    "Found duplicate keys, please make all workspaces and module names unique: {}",
                    duplicate_keys_string(error)
                )
            }
            Error::Custom(c) => write!(f, "{}", c),
            Error::DirectoryDoesNotExist(p) => write!(f, "Directory does not exists: '{}'", p),
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
