use std::collections::BTreeSet;

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Workspace {
    pub name: String,
    pub modules: BTreeSet<String>,
}
