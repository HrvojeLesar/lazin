use std::{
    collections::BTreeSet,
    fmt::Display,
    path::{Path, PathBuf},
};

use lazin_error::{Context, LazinResult};
use lazin_pipeline::Bind;

use crate::{
    config::{self, Name},
    encryption_management::{EncryptionManager, GPG_EXTENSION},
    error::LazinError,
    resolve::{self},
};

pub enum ValidationError {
    SourcePathDoesNotExist { module_name: Name, path: PathBuf },
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::SourcePathDoesNotExist { module_name, path } => {
                write!(
                    f,
                    "Module '{}' has a source path that could not be found: '{}'",
                    module_name,
                    path.display()
                )
            }
        }
    }
}

#[derive(Debug)]
pub struct Config {
    pub expanded_modules: BTreeSet<resolve::module::Module>,
    pub workspaces: BTreeSet<resolve::workspace::Workspace>,
    pub encryption_manager: EncryptionManager,
}

impl Config {
    pub fn parse(
        config: config::config::Config,
        encryption_manager: EncryptionManager,
    ) -> LazinResult<Self> {
        let validated_modules =
            validate_module_sources(config.modules).context("Failed to validate module sources")?;

        let expanded_modules = validated_modules
            .into_iter()
            .map(|m| resolve::module::Module::try_new(m))
            .collect::<LazinResult<BTreeSet<resolve::module::Module>>>()?;

        let workspaces = config
            .workspaces
            .into_iter()
            .map(|w| {
                let name: String = w.name.into();
                let modules = w
                    .modules
                    .into_iter()
                    .map(|module_name| {
                        let module = expanded_modules
                            .iter()
                            .find(|m| m.name == module_name.as_ref());
                        match module {
                            Some(m) => Ok(m.name.clone()),
                            None => Err(LazinError::ModuleNotFound(
                                name.clone().into(),
                                module_name.clone(),
                            ))
                            .context("Module not found"),
                        }
                    })
                    .collect::<LazinResult<BTreeSet<String>>>()?;
                Ok(resolve::workspace::Workspace { name, modules })
            })
            .collect::<LazinResult<BTreeSet<resolve::workspace::Workspace>>>()?;

        Ok(Self {
            expanded_modules,
            workspaces,
            encryption_manager,
        })
    }

    pub fn contains_workspace(&self, name: &str) -> bool {
        self.workspaces.iter().any(|w| w.name == name)
    }

    pub fn get_workspace_modules(&self, name: &str) -> Vec<&resolve::module::Module> {
        let workspace = match self.workspaces.iter().find(|w| w.name == name) {
            Some(w) => w,
            None => return Vec::new(),
        };

        workspace.modules.iter().fold(
            Vec::new(),
            |mut acc: Vec<&resolve::module::Module>, module_name| {
                if let Some(m) = self
                    .expanded_modules
                    .iter()
                    .find(|m| m.name == *module_name)
                {
                    acc.push(m)
                };

                acc
            },
        )
    }
}

fn validate_module_sources(
    modules: BTreeSet<config::module::Module>,
) -> LazinResult<BTreeSet<config::module::Module>> {
    modules.iter().try_for_each(|module| {
        match module
            .values
            .iter()
            .map(|(source_path, value)| match &value.config.encryption {
                config::module::Encryption::Disabled => {
                    unencrypted_source_path_validation_pipeline(module, source_path)
                }
                config::module::Encryption::Enabled { .. } => {
                    encrypted_source_path_validation_pipeline(module, source_path)
                }
            })
            .fold(true, |acc, result| match result {
                Ok(_) => acc,
                Err(e) => {
                    lazin_logger::error!(e);
                    false
                }
            }) {
            true => Ok(()),
            false => Err(LazinError::InvalidModuleSources),
        }
    })?;

    Ok(modules)
}

fn unencrypted_source_path_validation_pipeline(
    module: &config::module::Module,
    source_path: &config::module::SourcePath,
) -> Result<(), ValidationError> {
    lazin_pipeline::new(source_path)
        .bind(|path| {
            let path = Path::new(path.as_ref());
            match path.exists() {
                true => Ok(()),
                false => Err(ValidationError::SourcePathDoesNotExist {
                    module_name: module.name.clone(),
                    path: path.into(),
                }),
            }
        })
        .result()
}

fn encrypted_source_path_validation_pipeline(
    module: &config::module::Module,
    source_path: &config::module::SourcePath,
) -> Result<(), ValidationError> {
    let path = Path::new(source_path.as_ref());
    if path.is_dir() {
        lazin_pipeline::new(source_path).result()
    } else {
        lazin_pipeline::new(source_path)
            .bind(|path| {
                let path = Path::new(path.as_ref());
                let encrypted_file_path = path.with_added_extension(GPG_EXTENSION);
                match (path.exists(), encrypted_file_path.exists()) {
                    (true, true) | (true, false) | (false, true) => Ok(()),
                    (false, false) => Err(ValidationError::SourcePathDoesNotExist {
                        module_name: module.name.clone(),
                        path: path.into(),
                    }),
                }
            })
            .result()
    }
}
