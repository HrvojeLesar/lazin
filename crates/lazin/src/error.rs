use std::{fmt::Display, path::Path};

use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFile,
    term::{self, termcolor::Buffer},
};

use crate::common::Key;

pub type DuplicateKeys = Vec<Key>;

pub struct DuplicateKeysError {
    pub duplicates: DuplicateKeys,
}

pub type LazinError = Error;
pub type LazinResult<T> = Result<T, LazinError>;

#[derive(Debug)]
pub struct TomlError {
    pub filename: String,
    pub source: String,
    pub error: toml::de::Error,
}

impl Display for TomlError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let span = self.error.span().expect("a valid span");
        let file = SimpleFile::new(self.filename.as_str(), self.source.as_str());

        let diagnostic = Diagnostic::error()
            .with_message(self.error.message())
            .with_label(Label::primary((), span));

        let config = codespan_reporting::term::Config::default();
        let mut buffer = Buffer::ansi();

        term::emit_to_write_style(&mut buffer, &config, &file, &diagnostic)
            .expect("a valid error emit");

        let rendered = String::from_utf8(buffer.into_inner()).map_err(|_| std::fmt::Error)?;
        write!(f, "{}", rendered)
    }
}

#[derive(Debug)]
pub enum Error {
    Toml(Box<TomlError>),
    Io(std::io::Error),
    DuplicateKeys(DuplicateKeys),
    DirectoryDoesNotExist(String),
    Custom(&'static str),
    InvalidModuleSources,
    ModuleNotFound(String),
}

impl Error {
    pub fn directory_does_not_exist(path: &Path) -> Self {
        Self::DirectoryDoesNotExist(format!("{}", path.display()))
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Toml(error) => error.fmt(f),
            Error::Io(error) => error.fmt(f),
            Error::DuplicateKeys(error) => {
                write!(
                    f,
                    "Found duplicate keys, please make all workspaces and module names unique. Duplicate names: {}",
                    duplicate_keys_string(error)
                )
            }
            Error::Custom(c) => write!(f, "{}", c),
            Error::DirectoryDoesNotExist(p) => write!(f, "Directory does not exists: '{}'", p),
            Error::InvalidModuleSources => write!(f, "Invalid module sources"),
            Error::ModuleNotFound(m) => {
                write!(f, "Could not find a configured module named: '{}'", m)
            }
        }
    }
}

impl From<TomlError> for Error {
    fn from(value: TomlError) -> Self {
        Self::Toml(Box::new(value))
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

impl From<Vec<DuplicateKeysError>> for Error {
    fn from(value: Vec<DuplicateKeysError>) -> Self {
        let keys = value.into_iter().fold(Vec::new(), |mut acc, e| {
            acc.extend(e.duplicates);
            acc
        });
        Self::DuplicateKeys(keys)
    }
}

fn duplicate_keys_string(duplicates: &[Key]) -> String {
    duplicates
        .iter()
        .map(|dup| dup.str())
        .collect::<Vec<_>>()
        .join(", ")
}
