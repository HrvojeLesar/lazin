use std::{
    collections::{BTreeMap, btree_map},
    fs::File,
    io::Read,
    path::Path,
};

use lazin_error::{Context, LazinResult};
use serde::Deserialize;

use crate::{
    common::{self, Key, TomlFile},
    dotfiles::{module::config::ModuleConfig, workspace::RawWorkspace},
    error::{DuplicateKeysError, Error, TomlError},
};

type RawEntries = BTreeMap<Key, RawEntry>;

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RawEntry {
    Workspace(RawWorkspace),
    Module(ModuleConfig),
}

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    pub entries: RawEntries,
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
                File::open(&tomlfile.path)
                    .context("File error")?
                    .read_to_string(&mut file_data_buffer)
                    .context("File read to string error")?;
                match toml::from_str::<RawEntries>(&file_data_buffer) {
                    Ok(file_entires) => {
                        merge_entries(&mut entries, file_entires).map_err(Error::from)?
                    }
                    Err(e) => {
                        return Err(Error::from(TomlError {
                            filename: tomlfile.filename,
                            source: file_data_buffer.clone(),
                            error: e,
                        })
                        .into());
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
}
