use std::{fmt::Display, path::Path};

use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFile,
    term::{self, termcolor::Buffer},
};

use crate::{
    common::Key,
    dotfiles::{module::ModuleName, workspace::WorkspaceName},
};

pub type DuplicateKeys = Vec<Key>;

pub struct DuplicateKeysError {
    pub duplicates: DuplicateKeys,
}

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
pub enum LazinError {
    Toml(Box<TomlError>),
    Io(std::io::Error),
    DuplicateKeys(DuplicateKeys),
    DirectoryDoesNotExist(String),
    Custom(&'static str),
    InvalidModuleSources,
    ModuleNotFound(WorkspaceName, ModuleName),
    WorkspaceNotFound(Key),
    StripPrefix(std::path::StripPrefixError),
    GpgWrapper(lazin_gpg_wrapper::Error),
}

impl LazinError {
    pub fn directory_does_not_exist(path: &Path) -> Self {
        Self::DirectoryDoesNotExist(format!("{}", path.display()))
    }
}

impl Display for LazinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LazinError::Toml(error) => write!(f, "Toml error: {}", error),
            LazinError::Io(error) => write!(f, "Io error: {}", error),
            LazinError::DuplicateKeys(error) => {
                write!(
                    f,
                    "Found duplicate keys, please make all workspaces and module names unique. Duplicate names: {}",
                    duplicate_keys_string(error)
                )
            }
            LazinError::Custom(c) => write!(f, "{}", c),
            LazinError::DirectoryDoesNotExist(p) => write!(f, "Directory does not exists: '{}'", p),
            LazinError::InvalidModuleSources => write!(f, "Invalid module sources"),
            LazinError::ModuleNotFound(w, m) => {
                write!(
                    f,
                    "Workspace '{}' contains an unconfigured module '{}'",
                    w.as_ref(),
                    m.as_ref(),
                )
            }
            LazinError::WorkspaceNotFound(workspace) => {
                write!(f, "Could not find workspace: '{}'", workspace)
            }
            LazinError::StripPrefix(strip_prefix_error) => {
                write!(f, "Failed to strip prefix: {}", strip_prefix_error)
            }
            LazinError::GpgWrapper(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for LazinError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LazinError::Toml(error) => error.error.source(),
            LazinError::Io(error) => error.source(),
            LazinError::DuplicateKeys(_) => None,
            LazinError::Custom(_) => None,
            LazinError::DirectoryDoesNotExist(_) => None,
            LazinError::InvalidModuleSources => None,
            LazinError::ModuleNotFound(_, _) => None,
            LazinError::WorkspaceNotFound(_) => None,
            LazinError::StripPrefix(strip_prefix_error) => strip_prefix_error.source(),
            LazinError::GpgWrapper(error) => error.source(),
        }
    }
}

impl From<TomlError> for LazinError {
    fn from(value: TomlError) -> Self {
        Self::Toml(Box::new(value))
    }
}

impl From<DuplicateKeysError> for LazinError {
    fn from(value: DuplicateKeysError) -> Self {
        Self::DuplicateKeys(value.duplicates)
    }
}

impl From<Vec<DuplicateKeysError>> for LazinError {
    fn from(value: Vec<DuplicateKeysError>) -> Self {
        let keys = value.into_iter().fold(Vec::new(), |mut acc, e| {
            acc.extend(e.duplicates);
            acc
        });
        Self::DuplicateKeys(keys)
    }
}

impl From<std::io::Error> for LazinError {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

impl From<std::path::StripPrefixError> for LazinError {
    fn from(value: std::path::StripPrefixError) -> Self {
        Self::StripPrefix(value)
    }
}

impl From<lazin_gpg_wrapper::Error> for LazinError {
    fn from(value: lazin_gpg_wrapper::Error) -> Self {
        Self::GpgWrapper(value)
    }
}

fn duplicate_keys_string(duplicates: &[Key]) -> String {
    duplicates
        .iter()
        .map(|dup| dup.str())
        .collect::<Vec<_>>()
        .join(", ")
}
