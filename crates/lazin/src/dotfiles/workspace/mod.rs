use crate::common::Key;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
#[serde(transparent)]
pub struct Workspace {
    #[serde(flatten)]
    modules: Vec<Key>,
}
