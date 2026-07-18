//! This module defines structs that are used to configure `lazin`.

use std::fmt::Display;

use serde::Deserialize;

pub(crate) mod config;
pub(crate) mod module;
pub(crate) mod workspace;

#[derive(Clone, Deserialize, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Name(String);

impl From<Name> for String {
    fn from(value: Name) -> Self {
        value.0
    }
}

impl AsRef<str> for Name {
    fn as_ref(&self) -> &str {
        &self.0
    }
}

impl Display for Name {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0)
    }
}

impl From<String> for Name {
    fn from(value: String) -> Self {
        Self(value)
    }
}
