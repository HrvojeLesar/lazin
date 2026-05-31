use std::fmt::Display;

use crate::dotfiles;

#[derive(Debug)]
pub enum Error {
    DotfilesError(dotfiles::error::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::DotfilesError(error) => error.fmt(f)
        }
    }
}

impl From<dotfiles::error::Error> for Error {
    fn from(value: dotfiles::error::Error) -> Self {
        Self::DotfilesError(value)
    }
}
