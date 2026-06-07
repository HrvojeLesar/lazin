use crate::{
    common::{self, parse_config},
    error::LazinResult,
};

use clap::Args;
use std::path::PathBuf;

/// Check configuration validity
#[derive(Args)]
pub(super) struct Check {
    #[arg(short = 'd', long = "directory", help = "directory to validate")]
    directory: Option<PathBuf>,
}

impl Check {
    pub(crate) fn check(&self) -> LazinResult<()> {
        let config = parse_config(self.directory.as_deref())?;

        let validation_errors = common::validate_config(&config);

        if !validation_errors.is_empty() {
            common::report_validation_errors(validation_errors);
            common::exit_error()
        }

        Ok(())
    }
}
