use crate::common::{self};

use clap::Args;
use lazin_error::LazinResult;
use std::path::PathBuf;

/// Check configuration validity
#[derive(Args)]
pub(super) struct Check {
    #[arg(short = 'd', long = "directory", help = "Directory to validate")]
    directory: Option<PathBuf>,
    #[arg(
        short = 'g',
        long = "gitignore",
        help = "Gitignore file, can be a non existing file to not use gitignore at all. By default looks for .gitignore in the current working directory"
    )]
    gitignore: Option<PathBuf>,
}

impl Check {
    pub(crate) fn check(&self) -> LazinResult<()> {
        common::parse_config(self.directory.as_deref(), self.gitignore.as_deref())?;

        lazin_logger::info!("Configuration is valid");

        Ok(())
    }
}
