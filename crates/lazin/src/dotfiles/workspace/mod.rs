use crate::common::Key;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct RawWorkspace {
    modules: Vec<Key>,
}

impl RawWorkspace {
    pub fn modules(&self) -> &[Key] {
        &self.modules
    }
}

#[derive(Clone, Debug)]
pub struct Workspace {
    modules: Vec<Key>,
}

impl From<RawWorkspace> for Workspace {
    fn from(value: RawWorkspace) -> Self {
        Self {
            modules: value.modules,
        }
    }
}
