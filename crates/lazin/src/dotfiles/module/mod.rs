use lazin_error::{Context, LazinResult};

use crate::{
    common::Key,
    dotfiles::module::config::{ModuleConfig, ModuleConfigValue},
    error::LazinError,
};
use std::{
    borrow::Borrow,
    collections::BTreeSet,
    env,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
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
pub struct EncryptionManagement {
    pub manage_encryption: bool,
    pub recipient: Rc<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ModuleValue {
    pub source: PathBuf,
    pub target: PathBuf,
    pub encryption: EncryptionManagement,
}

impl ModuleValue {
    pub fn try_new(
        source: &Path,
        module_config_value: &ModuleConfigValue,
        module_config: &ModuleConfig,
        recipient: Rc<String>,
    ) -> LazinResult<Self> {
        Ok(Self {
            source: source.into(),
            target: expand_tilde(module_config_value.path())?,
            encryption: EncryptionManagement {
                manage_encryption: module_config_value.is_encrypted(module_config),
                recipient,
            },
        })
    }
}

#[derive(Debug, Clone)]
pub struct Module {
    pub name: ModuleName,
    pub values: BTreeSet<ModuleValue>,
    pub manage_encryption: Option<bool>,
}

impl Module {
    pub fn parse(name: &Key, module_config: &ModuleConfig) -> LazinResult<Self> {
        let mut values = BTreeSet::new();
        for (source, target) in &module_config.values {
            let source = Path::new(source.0.str());
            let expanded =
                expand_directory(source, target, module_config, Rc::new("".to_string()))?;
            values.extend(expanded);
        }

        Ok(Self {
            name: ModuleName(name.clone()),
            values,
            manage_encryption: module_config.encrypt,
        })
    }
}

fn expand_directory(
    source: &Path,
    module_config_value: &ModuleConfigValue,
    module_config: &ModuleConfig,
    recipient: Rc<String>,
) -> LazinResult<BTreeSet<ModuleValue>> {
    fn walk(
        source: &Path,
        module_config_value: &ModuleConfigValue,
        out: &mut BTreeSet<ModuleValue>,
        module_config: &ModuleConfig,
        recipient: Rc<String>,
    ) -> LazinResult<()> {
        if !source.is_dir() {
            out.insert(ModuleValue::try_new(
                source,
                module_config_value,
                module_config,
                recipient,
            )?);
        } else {
            for child in fs::read_dir(source).context("Failed to read child directory")? {
                let child = child.context("Failed to get child directory")?.file_name();
                let child_source = source.join(&child);
                let child_target = module_config_value.join(&child);
                walk(
                    &child_source,
                    &child_target,
                    out,
                    module_config,
                    recipient.clone(),
                )?;
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
        recipient,
    )?;
    Ok(module_values)
}

fn expand_tilde(path: &Path) -> LazinResult<PathBuf> {
    if !path.starts_with("~") {
        return Ok(path.into());
    }

    let home_dir = env::var_os("HOME").ok_or(LazinError::Custom(
        "unable to determine HOME directory; Lazin cannot run without detecting the home directory",
    ))?;
    let home_dir = PathBuf::from(home_dir);
    let stripped_path = path
        .strip_prefix("~")
        .context("Failed to strip tilde prefix")?;

    Ok(home_dir.join(stripped_path))
}
