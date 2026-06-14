use crate::common::Key;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct RawWorkspace {
    pub modules: Vec<Key>,
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
