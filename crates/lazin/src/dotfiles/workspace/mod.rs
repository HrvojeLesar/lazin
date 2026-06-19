use std::{borrow::Borrow, fmt::Display, ops::Deref};

use crate::common::Key;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct RawWorkspace {
    pub modules: Vec<Key>,
}

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct WorkspaceName(pub Key);

impl Display for WorkspaceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Borrow<Key> for WorkspaceName {
    fn borrow(&self) -> &Key {
        &self.0
    }
}

impl AsRef<Key> for WorkspaceName {
    fn as_ref(&self) -> &Key {
        &self.0
    }
}

#[derive(Clone, Debug)]
pub struct Workspace {
    pub name: WorkspaceName,
    pub modules: Vec<Key>,
}

impl Workspace {
    pub fn new(name: Key, modules: Vec<Key>) -> Self {
        Self {
            name: WorkspaceName(name),
            modules,
        }
    }
}
