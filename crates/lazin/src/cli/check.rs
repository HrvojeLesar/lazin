use crate::{common::parse_config, error::Error, validator::module::ModuleValidator};

use clap::Args;
use std::path::PathBuf;

/// Check configuration validity
#[derive(Args)]
pub(super) struct Check {
    #[arg(short = 'd', long = "directory", help = "directory to validate")]
    directory: Option<PathBuf>,
}

impl Check {
    pub(crate) fn check(&self) -> Result<(), Error> {
        let config = parse_config(self.directory.as_deref())?;

        let modules = config.modules();
        let mut validation_errors = Vec::new();
        for (module_name, module) in modules {
            validation_errors.extend(ModuleValidator::validate(module_name, module));
        }

        if !validation_errors.is_empty() {
            validation_errors
                .sort_by(|validation_a, validation_b| validation_a.key.cmp(validation_b.key));
            validation_errors.iter().for_each(|err| {
                lazin_logger::error!(
                    "Module '{}' path '{}': {}",
                    err.module_name.str(),
                    err.key.str(),
                    err.validation
                );
            });
        }

        Ok(())
    }
}
