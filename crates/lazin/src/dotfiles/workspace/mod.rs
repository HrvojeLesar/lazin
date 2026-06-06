use crate::common::Key;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(transparent)]
pub struct Workspace {
    modules: Vec<Key>,
}

impl Workspace {
    pub fn modules(&self) -> &[Key] {
        &self.modules
    }
}
