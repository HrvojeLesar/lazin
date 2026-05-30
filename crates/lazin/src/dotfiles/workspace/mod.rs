use serde::Deserialize;
use std::{collections::HashMap, convert::Into};

use crate::dotfiles::{common::Key, error::Error};

#[derive(Debug, Deserialize)]
pub struct Workspace {
    #[serde(flatten)]
    modules: HashMap<Key, Vec<Key>>,
}

impl Workspace {
    pub fn parse(input: &str) -> Result<Self, Error> {
        toml::from_str(input).map_err(|e| e.into())
    }
}

#[cfg(test)]
mod test {
    use crate::dotfiles::workspace::Workspace;

    #[test]
    fn parse_workspace() {
        Workspace::parse(
            r#"
        workspace1 = ["module1", "module2"]
        workspace2 = ["module1", "module2"]
        "#,
        )
        .expect("a valid toml");
    }
}
