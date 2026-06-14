use std::{
    collections::{BTreeMap, btree_map},
    fs::File,
    io::Read,
    path::Path,
};

use serde::Deserialize;

use crate::{
    common::{self, Key, TomlFile},
    dotfiles::{module::RawModule, workspace::RawWorkspace},
    error::{DuplicateKeysError, LazinError, LazinResult, TomlError},
};

type RawEntries = BTreeMap<Key, RawEntry>;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum RawEntry {
    Workspace(RawWorkspace),
    Module(RawModule),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    entries: RawEntries,
}

impl Config {
    pub fn parse(config_dir: &Path) -> LazinResult<Self> {
        fn merge_entries(
            entries: &mut RawEntries,
            other: RawEntries,
        ) -> Result<(), DuplicateKeysError> {
            let mut duplicate_keys_errors = Vec::new();

            for (key, module) in other {
                match entries.entry(key) {
                    btree_map::Entry::Occupied(entry) => {
                        duplicate_keys_errors.push(entry.key().clone());
                    }
                    btree_map::Entry::Vacant(entry) => {
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

        fn parse_entries(config_files: Vec<TomlFile>) -> LazinResult<RawEntries> {
            let mut entries = BTreeMap::new();
            let mut file_data_buffer = String::new();

            for tomlfile in config_files {
                file_data_buffer.clear();
                File::open(&tomlfile.path)?.read_to_string(&mut file_data_buffer)?;
                match toml::from_str::<RawEntries>(&file_data_buffer) {
                    Ok(file_entires) => merge_entries(&mut entries, file_entires)?,
                    Err(e) => {
                        return Err(LazinError::from(TomlError {
                            filename: tomlfile.filename,
                            source: file_data_buffer.clone(),
                            error: e,
                        }));
                    }
                }
            }

            Ok(entries)
        }

        let config_files = common::files(config_dir)?;
        let raw_entries = parse_entries(config_files)?;

        Ok(Self {
            entries: raw_entries,
        })
    }

    pub fn workspaces(&self) -> Vec<(&Key, &RawWorkspace)> {
        self.entries
            .iter()
            .filter_map(|entry| match entry.1 {
                RawEntry::Workspace(w) => Some((entry.0, w)),
                RawEntry::Module(_) => None,
            })
            .collect()
    }

    pub fn modules(&self) -> Vec<(&Key, &RawModule)> {
        self.entries
            .iter()
            .filter_map(|entry| match entry.1 {
                RawEntry::Workspace(_) => None,
                RawEntry::Module(m) => Some((entry.0, m)),
            })
            .collect()
    }
}
