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

pub type LazinResult<T, C = &'static str> = Result<T, LazinContextError<C>>;

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
}

impl LazinError {
    pub fn directory_does_not_exist(path: &Path) -> Self {
        Self::DirectoryDoesNotExist(format!("{}", path.display()))
    }
}

// TODO: Better display for results
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

fn duplicate_keys_string(duplicates: &[Key]) -> String {
    duplicates
        .iter()
        .map(|dup| dup.str())
        .collect::<Vec<_>>()
        .join(", ")
}

#[derive(Debug)]
pub enum LazinContextError<C> {
    WithContext { context: C, error: LazinError },
    WithoutContext(LazinError),
}

impl<C: Display> From<LazinError> for LazinContextError<C> {
    fn from(value: LazinError) -> Self {
        Self::WithoutContext(value)
    }
}

impl<C: Display> Display for LazinContextError<C> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LazinContextError::WithContext { context, error } => {
                write!(f, "{}: {}", context, error)
            }
            LazinContextError::WithoutContext(error) => error.fmt(f),
        }
    }
}

pub trait Context<T> {
    fn context<C: Display>(self, context: C) -> Result<T, LazinContextError<C>>;
    fn with_context<C: Display, F: FnOnce() -> C>(
        self,
        context: F,
    ) -> Result<T, LazinContextError<C>>;
}

impl<T, E> Context<T> for Result<T, E>
where
    E: Into<LazinError>,
{
    fn context<C: Display>(self, context: C) -> Result<T, LazinContextError<C>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(LazinContextError::WithContext {
                context,
                error: error.into(),
            }),
        }
    }

    fn with_context<C: Display, F: FnOnce() -> C>(
        self,
        context: F,
    ) -> Result<T, LazinContextError<C>> {
        match self {
            Ok(ok) => Ok(ok),
            Err(error) => Err(LazinContextError::WithContext {
                context: context(),
                error: error.into(),
            }),
        }
    }
}

impl<T> Context<T> for LazinError {
    fn context<C: Display>(self, context: C) -> Result<T, LazinContextError<C>> {
        Err(LazinContextError::WithContext {
            context,
            error: self,
        })
    }

    fn with_context<C: Display, F: FnOnce() -> C>(
        self,
        context: F,
    ) -> Result<T, LazinContextError<C>> {
        Err(LazinContextError::WithContext {
            context: context(),
            error: self,
        })
    }
}
