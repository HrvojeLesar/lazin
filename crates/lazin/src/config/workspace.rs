use std::ops::Deref;

use serde::Deserialize;

use crate::config::{Name, module};

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Modules(Vec<module::SourcePath>);

impl Deref for Modules {
    type Target = Vec<module::SourcePath>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Workspace {
    pub name: Name,
    pub modules: Modules,
}
