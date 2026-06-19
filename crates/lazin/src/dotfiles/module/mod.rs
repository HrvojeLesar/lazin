use crate::{common::Key, error::LazinResult};
use serde::Deserialize;
use std::{
    borrow::Borrow,
    collections::BTreeMap,
    fmt::Display,
    fs,
    ops::Deref,
    path::{Path, PathBuf},
};

type FileSourceTargetPairs = BTreeMap<PathBuf, PathBuf>;

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

#[derive(Clone, Debug, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleSourcePath(pub Key);

impl Display for ModuleSourcePath {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

#[derive(Debug, Deserialize)]
pub struct RawModule {
    #[serde(flatten)]
    pub values: BTreeMap<ModuleSourcePath, ModuleValue>,
    #[serde(default)]
    pub encrypt: bool,
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: ModuleName,
    pub values: BTreeMap<PathBuf, PathBuf>,
    pub encrypt: bool,
}

impl Module {
    // TODO: support encryption
    pub fn parse(name: &Key, raw_module: &RawModule) -> LazinResult<Self> {
        let mut values = BTreeMap::new();
        for (source, target) in &raw_module.values {
            let source = Path::new(source.0.str());
            let expanded = expand_directory(source, target.path())?;
            values.extend(expanded);
        }

        Ok(Self {
            name: ModuleName(name.clone()),
            values,
            encrypt: raw_module.encrypt,
        })
    }
}

fn expand_directory(source: &Path, target: &Path) -> LazinResult<FileSourceTargetPairs> {
    fn walk(source: &Path, target: &Path, out: &mut FileSourceTargetPairs) -> LazinResult<()> {
        if !source.is_dir() {
            out.insert(source.into(), target.into());
        } else {
            for child in fs::read_dir(source)? {
                let child = child?.file_name();
                let child_source = source.join(&child);
                let child_target = target.join(&child);
                walk(&child_source, &child_target, out)?;
            }
        }
        Ok(())
    }

    let mut pairs = FileSourceTargetPairs::new();
    walk(source, target, &mut pairs)?;
    Ok(pairs)
}
