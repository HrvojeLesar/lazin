use crate::{common::parse_config, error::Error};

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
        parse_config(self.directory.as_deref())?;

        Ok(())
    }
}
