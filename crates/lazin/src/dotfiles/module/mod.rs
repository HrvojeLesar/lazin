use crate::{
    common::Key,
    dotfiles::module::config::{ModuleConfig, ModuleConfigValue},
    error::{LazinError, LazinResult},
};
use std::{
    borrow::Borrow,
    collections::BTreeSet,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

pub mod config;

#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleName(pub Key);

impl Display for ModuleName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl Borrow<Key> for ModuleName {
    fn borrow(&self) -> &Key {
        &self.0
    }
}

impl AsRef<Key> for ModuleName {
    fn as_ref(&self) -> &Key {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleValue {
    pub source: PathBuf,
    pub target: PathBuf,
    pub encrypt: bool,
}

impl ModuleValue {
    pub fn new(
        source: &Path,
        module_config_value: &ModuleConfigValue,
        module_config: &ModuleConfig,
    ) -> Self {
        Self {
            source: source.into(),
            target: module_config_value.path().into(),
            encrypt: module_config_value.is_encrypted(module_config),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: ModuleName,
    pub values: BTreeSet<ModuleValue>,
    pub encrypt: Option<bool>,
}

impl Module {
    pub fn parse(name: &Key, module_config: &ModuleConfig) -> LazinResult<Self> {
        let mut values = BTreeSet::new();
        for (source, target) in &module_config.values {
            let source = Path::new(source.0.str());
            let expanded = expand_directory(source, target, module_config)?;
            values.extend(expanded);
        }

        Ok(Self {
            name: ModuleName(name.clone()),
            values,
            encrypt: module_config.encrypt,
        })
    }
}

fn expand_directory(
    source: &Path,
    module_config_value: &ModuleConfigValue,
    module_config: &ModuleConfig,
) -> LazinResult<BTreeSet<ModuleValue>> {
    fn walk(
        source: &Path,
        module_config_value: &ModuleConfigValue,
        out: &mut BTreeSet<ModuleValue>,
        module_config: &ModuleConfig,
    ) -> LazinResult<()> {
        if !source.is_dir() {
            out.insert(ModuleValue::new(source, module_config_value, module_config));
        } else {
            for child in fs::read_dir(source)
                .map_err(|e| LazinError::IoExt("Failed to read child dir", e))?
            {
                let child = child
                    .map_err(|e| LazinError::IoExt("Failed to get child directory", e))?
                    .file_name();
                let child_source = source.join(&child);
                let child_target = module_config_value.join(&child);
                walk(&child_source, &child_target, out, module_config)?;
            }
        }
        Ok(())
    }

    let mut module_values = BTreeSet::new();
    walk(
        source,
        module_config_value,
        &mut module_values,
        module_config,
    )?;
    Ok(module_values)
}
