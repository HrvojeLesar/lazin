use std::{fmt::Display, path::Path};

use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFile,
    term::{self, termcolor::Buffer},
};

use crate::config::{self, Name};

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
    DirectoryDoesNotExist(String),
    Custom(&'static str),
    InvalidModuleSources,
    ModuleNotFound(Name, Name),
    StripPrefix(std::path::StripPrefixError),
    GpgWrapper(lazin_gpg_wrapper::Error),
    DuplicateKeys(config::DuplicateKeysError),
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
            LazinError::StripPrefix(strip_prefix_error) => {
                write!(f, "Failed to strip prefix: {}", strip_prefix_error)
            }
            LazinError::GpgWrapper(error) => error.fmt(f),
            LazinError::DuplicateKeys(error) => {
                let messages: Vec<String> = [
                    (!error.workspaces.is_empty()).then(|| format!(
                        "Found duplicate workspace names, please make sure all workspaces names are unique. Duplicate names: {}",
                        error.workspaces.join(", ")
                    )),
                    (!error.modules.is_empty()).then(|| format!(
                        "Found duplicate module names, please make sure all module names are unique. Duplicate names: {}",
                        error.modules.join(", ")
                    )),
                ]
                    .into_iter()
                    .flatten()
                    .collect();

                write!(f, "{}", messages.join("\n"))
            }
        }
    }
}

impl std::error::Error for LazinError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            LazinError::Toml(error) => error.error.source(),
            LazinError::Io(error) => error.source(),
            LazinError::Custom(_) => None,
            LazinError::DirectoryDoesNotExist(_) => None,
            LazinError::InvalidModuleSources => None,
            LazinError::ModuleNotFound(_, _) => None,
            LazinError::StripPrefix(strip_prefix_error) => strip_prefix_error.source(),
            LazinError::GpgWrapper(error) => error.source(),
            LazinError::DuplicateKeys(_) => None,
        }
    }
}

impl From<TomlError> for LazinError {
    fn from(value: TomlError) -> Self {
        Self::Toml(Box::new(value))
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

impl From<config::DuplicateKeysError> for LazinError {
    fn from(value: config::DuplicateKeysError) -> Self {
        Self::DuplicateKeys(value)
    }
}
