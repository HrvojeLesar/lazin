use crate::common::Key;
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub struct ModuleCompositeValue {
    path: PathBuf,
    #[serde(default)]
    encrypt: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ModuleValue {
    InlinePath(PathBuf),
    CompositeValue(ModuleCompositeValue),
}

impl ModuleValue {
    pub fn is_encrypted(&self) -> bool {
        match self {
            ModuleValue::CompositeValue(module_composite_value) => module_composite_value.encrypt,
            _ => false,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            ModuleValue::InlinePath(path_buf) => path_buf.as_path(),
            ModuleValue::CompositeValue(module_composite_value) => {
                module_composite_value.path.as_path()
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct RawModule {
    #[serde(flatten)]
    values: BTreeMap<Key, ModuleValue>,
    #[serde(default)]
    encrypt: bool,
}

impl RawModule {
    pub fn values_pairs(&self) -> impl Iterator<Item = (&Key, &ModuleValue)> {
        self.values.iter()
    }
}

#[derive(Debug)]
pub struct Module {
    values: BTreeMap<Key, ModuleValue>,
    encrypt: bool,
}

impl From<RawModule> for Module {
    fn from(value: RawModule) -> Self {
        todo!()
    }
}
