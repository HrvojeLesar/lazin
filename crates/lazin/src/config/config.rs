use core::fmt;
use std::{
    collections::{BTreeMap, BTreeSet},
    fs::File,
    io::Read,
    path::Path,
};

use lazin_error::{Context, LazinResult};
use serde::{
    Deserialize, Deserializer,
    de::{DeserializeSeed, MapAccess, Visitor},
};

use crate::{
    config::{Name, module, workspace},
    error::{LazinError, TomlError},
};

#[derive(Debug)]
pub struct Config {
    pub modules: BTreeSet<module::Module>,
    pub workspaces: BTreeSet<workspace::Workspace>,
}

#[derive(Debug, Default)]
pub struct DuplicateKeysError {
    pub workspaces: Vec<String>,
    pub modules: Vec<String>,
}

impl DuplicateKeysError {
    fn is_empty(&self) -> bool {
        self.workspaces.is_empty() && self.modules.is_empty()
    }
}

impl Config {
    pub fn parse(config_dir: &Path) -> LazinResult<Self> {
        #[derive(Debug, Deserialize)]
        #[serde(untagged)]
        pub enum RawEntry {
            Workspace(workspace::Workspace),
            Module(module::Module),
        }

        struct RawEntrySeed<'a> {
            key: &'a str,
        }

        impl<'de, 'a> DeserializeSeed<'de> for RawEntrySeed<'a> {
            type Value = RawEntry;

            fn deserialize<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer<'de>,
            {
                let mut value = toml::Value::deserialize(deserializer)?;

                match &mut value {
                    toml::Value::Table(table) => {
                        table.insert(
                            "name".to_string(),
                            toml::Value::String(self.key.to_string()),
                        );
                        let module =
                            module::Module::deserialize(value).map_err(serde::de::Error::custom)?;
                        Ok(RawEntry::Module(module))
                    }
                    toml::Value::Array(_) => {
                        let modules = workspace::Modules::deserialize(value)
                            .map_err(serde::de::Error::custom)?;
                        Ok(RawEntry::Workspace(workspace::Workspace {
                            name: Name(self.key.to_string()),
                            modules,
                        }))
                    }
                    other => Err(serde::de::Error::custom(format!(
                        "expected `{}` to be a module table or workspace array, found {}",
                        self.key,
                        other.type_str()
                    ))),
                }
            }
        }

        fn merge_entries(config: &mut Config, other: RawEntries) -> Result<(), DuplicateKeysError> {
            let mut duplicate_keys_errors = DuplicateKeysError::default();

            for (key, module) in other.0 {
                match module {
                    RawEntry::Workspace(workspace) => {
                        if config.workspaces.iter().any(|w| w.name == workspace.name) {
                            duplicate_keys_errors.workspaces.push(key);
                        } else {
                            config.workspaces.insert(workspace);
                        }
                    }
                    RawEntry::Module(module) => {
                        if config.modules.iter().any(|w| w.name == module.name) {
                            duplicate_keys_errors.modules.push(key);
                        } else {
                            config.modules.insert(module);
                        }
                    }
                }
            }

            if !duplicate_keys_errors.is_empty() {
                return Err(duplicate_keys_errors);
            }

            Ok(())
        }

        struct RawEntries(BTreeMap<String, RawEntry>);

        impl<'de> Deserialize<'de> for RawEntries {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct RawEntriesVisitor;

                impl<'de> Visitor<'de> for RawEntriesVisitor {
                    type Value = BTreeMap<String, RawEntry>;

                    fn expecting(&self, f: &mut fmt::Formatter) -> fmt::Result {
                        f.write_str("a table of workspaces or modules")
                    }

                    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
                    where
                        A: MapAccess<'de>,
                    {
                        let mut out = BTreeMap::new();
                        while let Some(key) = map.next_key::<String>()? {
                            let entry = map.next_value_seed(RawEntrySeed { key: &key })?;
                            out.insert(key, entry);
                        }
                        Ok(out)
                    }
                }

                deserializer
                    .deserialize_map(RawEntriesVisitor)
                    .map(RawEntries)
            }
        }

        let config_files = crate::common::files(config_dir)?;
        let mut config = Config {
            modules: BTreeSet::new(),
            workspaces: BTreeSet::new(),
        };

        let mut file_data_buffer = String::new();
        for file in config_files {
            file_data_buffer.clear();
            File::open(file.path)
                .context("Failed to open config toml file")?
                .read_to_string(&mut file_data_buffer)
                .context("Failed to read toml file into string")?;

            match toml::from_str::<RawEntries>(&file_data_buffer) {
                Ok(file_entires) => merge_entries(&mut config, file_entires)
                    .map_err(LazinError::from)
                    .context("Failed to merge config and file entries")?,
                Err(e) => {
                    return Err(LazinError::from(TomlError {
                        filename: file.filename,
                        source: file_data_buffer.clone(),
                        error: e,
                    }))
                    .context("Failed to parse config files")?;
                }
            }
        }

        Ok(config)
    }
}
