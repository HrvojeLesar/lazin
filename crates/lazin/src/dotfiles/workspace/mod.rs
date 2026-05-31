use serde::Deserialize;
use std::{
    collections::{HashMap, hash_map::Entry},
    convert::Into,
};

use crate::{
    common::Key,
    error::{DuplicateKeysError, Error},
};

#[derive(Debug, Deserialize)]
pub struct Workspace {
    #[serde(flatten)]
    modules: HashMap<Key, Vec<Key>>,
}

impl Workspace {
    pub fn parse(input: &str) -> Result<Self, Error> {
        toml::from_str(input).map_err(|e| e.into())
    }

    pub fn empty() -> Self {
        Self {
            modules: HashMap::default(),
        }
    }

    pub fn join(&mut self, other: Workspace) -> Result<(), DuplicateKeysError> {
        let mut duplicate_keys_errors = Vec::new();

        for (key, module) in other.modules {
            match self.modules.entry(key) {
                Entry::Occupied(entry) => {
                    duplicate_keys_errors.push(entry.key().clone());
                }
                Entry::Vacant(entry) => {
                    entry.insert(module);
                }
            }
        }

        if !duplicate_keys_errors.is_empty() {
            return Err(DuplicateKeysError {
                duplicates: duplicate_keys_errors,
            });
        }

        Ok(())
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
