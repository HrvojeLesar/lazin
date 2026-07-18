use std::collections::BTreeMap;

use serde::{Deserialize, Deserializer};

use crate::config::{Name, module};

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Modules(Vec<module::SourcePath>);

#[derive(Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct Workspace {
    pub name: Name,
    pub modules: Modules,
}
