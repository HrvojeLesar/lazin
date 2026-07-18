use serde::Deserialize;

use crate::config::Name;

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Workspace {
    pub name: Name,
    pub modules: Vec<Name>,
}
