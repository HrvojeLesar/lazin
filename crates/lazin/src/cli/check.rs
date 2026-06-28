use crate::common::{self};

use clap::Args;
use lazin_error::LazinResult;
use std::path::PathBuf;

/// Check configuration validity
#[derive(Args)]
pub(super) struct Check {
    #[arg(short = 'd', long = "directory", help = "directory to validate")]
    directory: Option<PathBuf>,
}

impl Check {
    pub(crate) fn check(&self) -> LazinResult<()> {
        common::check(self.directory.as_deref())?;

        Ok(())
    }
}
