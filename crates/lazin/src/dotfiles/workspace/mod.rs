use std::fmt::Display;

use crate::common::Key;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct RawWorkspace {
    pub modules: Vec<Key>,
}

#[derive(Clone, Debug)]
pub struct WorkspaceName(pub Key);

impl Display for WorkspaceName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Clone, Debug)]
pub struct Workspace {
    pub name: Key,
    pub modules: Vec<Key>,
}

impl Workspace {
    pub fn new(name: Key, modules: Vec<Key>) -> Self {
        Self { name, modules }
    }
}
