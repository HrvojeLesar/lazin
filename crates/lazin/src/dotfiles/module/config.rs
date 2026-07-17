use crate::common::Key;
use serde::Deserialize;
use std::{
    collections::BTreeMap,
    fmt::Display,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub struct ModuleConfigCompositeValue {
    path: PathBuf,
    encrypt: Option<bool>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ModuleConfigValue {
    InlinePath(PathBuf),
    CompositeValue(ModuleConfigCompositeValue),
}

impl ModuleConfigValue {
    pub fn is_encrypted(&self, module: &ModuleConfig) -> bool {
        let module_level_encryption = module.encrypt.unwrap_or_default();

        match self {
            ModuleConfigValue::CompositeValue(module_composite_value) => module_composite_value
                .encrypt
                .unwrap_or(module_level_encryption),
            _ => module_level_encryption,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            ModuleConfigValue::InlinePath(path_buf) => path_buf.as_path(),
            ModuleConfigValue::CompositeValue(module_composite_value) => {
                module_composite_value.path.as_path()
            }
        }
    }

    pub fn join<P: AsRef<Path>>(&self, path: P) -> Self {
        let target_path = self.path().join(path);
        match self {
            ModuleConfigValue::InlinePath(_) => Self::InlinePath(target_path),
            ModuleConfigValue::CompositeValue(original) => {
                Self::CompositeValue(ModuleConfigCompositeValue {
                    path: target_path,
                    encrypt: original.encrypt,
                })
            }
        }
    }
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleSourcePath(pub Key);

impl Display for ModuleSourcePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Deserialize)]
pub struct ModuleConfig {
    #[serde(flatten)]
    pub values: BTreeMap<ModuleSourcePath, ModuleConfigValue>,
    // TODO: move into separate struct also add to ModuleConfigValue
    pub encrypt: Option<bool>,
    pub recipient: Option<String>,
}
