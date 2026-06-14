use crate::{common::Key, error::LazinResult};
use serde::Deserialize;
use std::{
    collections::{BTreeMap, HashMap},
    fs,
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
    name: Key,
    values: BTreeMap<PathBuf, PathBuf>,
    encrypt: bool,
}

impl Module {
    pub fn parse(name: &Key, raw_module: &RawModule) -> LazinResult<Self> {
        let mut values = BTreeMap::new();
        for (source, target) in &raw_module.values {
            let source = Path::new(source.str());
            let expanded = expand_directory(source, target.path())?;
            values.extend(expanded);
        }

        Ok(Self {
            name: name.clone(),
            values,
            encrypt: raw_module.encrypt,
        })
    }
}

fn expand_directory(source: &Path, target: &Path) -> LazinResult<FileSourceTargetPairs> {
    if !source.is_dir() {
        let pairs = FileSourceTargetPairs::from([(source.into(), target.into())]);
        Ok(pairs)
    } else {
        let expanded = fs::read_dir(source)?
            .map(|child| -> LazinResult<FileSourceTargetPairs> {
                let child = child?.file_name();
                let child_source = PathBuf::from(source).join(&child);
                let child_target = PathBuf::from(target).join(&child);
                expand_directory(&child_source, &child_target)
            })
            .collect::<LazinResult<Vec<FileSourceTargetPairs>>>()?; // Use transposition of Iterator<Result<T,E>> -> Result<Sequence<T>, E>
        Ok(expanded.into_iter().flatten().collect())
    }
}
