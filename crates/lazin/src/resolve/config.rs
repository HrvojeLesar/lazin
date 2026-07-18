use std::{collections::BTreeSet, path::Path};

use lazin_error::{Context, LazinResult};
use lazin_pipeline::Bind;

use crate::{
    config::{self},
    dotfiles::{
        self, module::ModuleName, resolved_config::ValidationError, workspace::WorkspaceName,
    },
    encryption_management::{EncryptionManager, GPG_EXTENSION},
    error::LazinError,
    resolve::{self},
};

#[derive(Debug)]
pub struct Config {
    pub expanded_modules: BTreeSet<resolve::module::Module>,
    pub workspaces: BTreeSet<resolve::workspace::Workspace>,
}

impl Config {
    pub fn parse(
        config: config::config::Config,
        mut encryption_manager: EncryptionManager,
    ) -> LazinResult<Self> {
        let validated_modules =
            validate_module_sources(config.modules).context("Failed to validate module sources")?;

        let expanded_modules = validated_modules
            .into_iter()
            .map(|m| resolve::module::Module::try_new(m, &mut encryption_manager))
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
                            None => {
                                let workspace_name_string: String = name.clone().into();
                                let module_name_string: String = module_name.into();

                                Err(LazinError::ModuleNotFound(
                                    WorkspaceName(workspace_name_string.into()),
                                    ModuleName(module_name_string.into()),
                                ))
                                .context("Module not found")
                            }
                        }
                    })
                    .collect::<LazinResult<BTreeSet<String>>>()?;
                Ok(resolve::workspace::Workspace { name, modules })
            })
            .collect::<LazinResult<BTreeSet<resolve::workspace::Workspace>>>()?;

        Ok(Self {
            expanded_modules,
            workspaces,
        })
    }
}

fn validate_module_sources(
    modules: BTreeSet<config::module::Module>,
) -> LazinResult<BTreeSet<config::module::Module>> {
    modules.iter().try_for_each(|module| {
        match module
            .values
            .keys()
            .map(|source_path| match &module.config.encryption {
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
) -> Result<(), dotfiles::resolved_config::ValidationError> {
    lazin_pipeline::new(source_path)
        .bind(|path| {
            let path = Path::new(path.as_ref());
            match path.exists() {
                true => Ok(()),
                false => {
                    // TODO: change error so it uses Name or T instead
                    let module_name_string: String = module.name.clone().into();
                    Err(ValidationError::SourcePathDoesNotExist {
                        module_name: module_name_string.into(),
                        path: path.into(),
                    })
                }
            }
        })
        .result()
}

fn encrypted_source_path_validation_pipeline(
    module: &config::module::Module,
    source_path: &config::module::SourcePath,
) -> Result<(), dotfiles::resolved_config::ValidationError> {
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
                    (false, false) => {
                        let module_name_string: String = module.name.clone().into();
                        Err(ValidationError::SourcePathDoesNotExist {
                            module_name: module_name_string.into(),
                            path: path.into(),
                        })
                    }
                }
            })
            .result()
    }
}
