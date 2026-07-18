use std::{
    collections::BTreeSet,
    env, fs,
    path::{Path, PathBuf},
};

use lazin_error::{Context, LazinResult};

use crate::{config, encryption_management::EncryptionManager, error::LazinError};

#[derive(Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum Encryption {
    #[default]
    Disabled,
    Enabled {
        recipient: String,
    },
}

impl From<config::module::Encryption> for Encryption {
    fn from(value: config::module::Encryption) -> Self {
        match value {
            config::module::Encryption::Disabled => Self::Disabled,
            config::module::Encryption::Enabled { recipient } => Self::Enabled { recipient },
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Value {
    pub source: PathBuf,
    pub target: PathBuf,
    pub encryption: Encryption,
}

impl Value {
    fn new(source: PathBuf, target: PathBuf, encryption: Encryption) -> Self {
        Self {
            source,
            target,
            encryption,
        }
    }
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Module {
    pub name: String,
    pub values: BTreeSet<Value>,
}

impl Module {
    pub fn try_new(
        module: config::module::Module,
        encryption_manager: &mut EncryptionManager,
    ) -> LazinResult<Self> {
        let values = module.values.into_iter().try_fold(
            BTreeSet::new(),
            |mut acc, (source, value)| -> LazinResult<BTreeSet<Value>> {
                expand_module(&mut acc, source.into(), value, encryption_manager)?;

                Ok(acc)
            },
        )?;

        Ok(Self {
            name: module.name.into(),
            values,
        })
    }
}

fn expand_module(
    buffer: &mut BTreeSet<Value>,
    source: PathBuf,
    value: config::module::Value,
    encryption_manager: &mut EncryptionManager,
) -> LazinResult {
    fn walk(
        buffer: &mut BTreeSet<Value>,
        source: PathBuf,
        target: PathBuf,
        config: &config::module::Config,
        encryption_manager: &mut EncryptionManager,
    ) -> LazinResult<()> {
        if !source.is_dir() {
            let target = expand_tilde(&target)?;
            match config.encryption {
                config::module::Encryption::Enabled { .. } => {
                    encryption_manager.manage_decryption(&source)?
                }
                _ => (),
            }
            buffer.insert(Value::new(source, target, config.encryption.clone().into()));
        } else {
            for child in fs::read_dir(&source).context("Failed to read child directory")? {
                let child = child.context("Failed to get child directory")?.file_name();
                let child_source = source.join(&child);
                let child_target = target.join(&child);
                walk(
                    buffer,
                    child_source,
                    child_target,
                    config,
                    encryption_manager,
                )?;
            }
        }
        Ok(())
    }

    walk(
        buffer,
        source,
        value.path,
        &value.config,
        encryption_manager,
    )
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
