use std::{collections::BTreeSet, path::Path};

use lazin_error::{Context, LazinResult};
use lazin_pipeline::Bind;

use crate::{
    config, dotfiles::resolved_config::ValidationError, encryption_management::EncryptionManager,
    error::LazinError, resolve,
};

#[derive(Debug)]
pub struct ResolvedConfig2 {
    pub expanded_modules: BTreeSet<resolve::module::Module>,
    pub workspaces: BTreeSet<resolve::workspace::Workspace>,
}

impl ResolvedConfig2 {
    pub fn parse(
        config: config::config::Config,
        encryption_manager: EncryptionManager,
    ) -> LazinResult<Self> {
        let modules =
            validate_module_sources(config.modules).context("Failed to validate module sources")?;

        todo!("end")
    }
}

fn validate_module_sources(
    modules: BTreeSet<config::module::Module>,
) -> LazinResult<BTreeSet<config::module::Module>> {
    modules.iter().try_for_each(|module| {
        match module
            .values
            .keys()
            .map(|source_path| {
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
