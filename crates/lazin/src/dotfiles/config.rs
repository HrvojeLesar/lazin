use std::collections::{HashMap, hash_map};

use serde::Deserialize;

use crate::{
    common::Key,
    dotfiles::{module::Module, workspace::Workspace},
    error::{DuplicateKeysError, Error},
};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    entries: HashMap<Key, Entry>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Entry {
    Workspace(Workspace),
    Module(Module),
}

impl Config {
    pub fn parse(input: &str) -> Result<Self, Error> {
        toml::from_str(input).map_err(Error::from)
    }

    pub fn empty() -> Self {
        Self {
            entries: HashMap::default(),
        }
    }

    pub fn join(&mut self, other: Self) -> Result<(), DuplicateKeysError> {
        let mut duplicate_keys_errors = Vec::new();

        for (key, module) in other.entries {
            match self.entries.entry(key) {
                hash_map::Entry::Occupied(entry) => {
                    duplicate_keys_errors.push(entry.key().clone());
                }
                hash_map::Entry::Vacant(entry) => {
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
