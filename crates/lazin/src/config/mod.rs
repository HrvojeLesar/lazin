//! This module defines structs that are used to configure `lazin`.

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
