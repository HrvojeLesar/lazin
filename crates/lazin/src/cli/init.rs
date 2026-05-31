use std::path::{Path, PathBuf};

use crate::dotfiles::filesystem::init::{init_default_config, init_directory};

use super::error::Error;
use clap::Args;

/// Initializes new `lazin` example configuration
///
/// Creates a directory with example configuration files.
#[derive(Args)]
pub(super) struct Init {
    #[arg(short = 'd', long = "directory")]
    directory: Option<PathBuf>,
}

impl Init {
    pub(crate) fn init(&self) -> Result<(), Error> {
        let directory = self
            .directory
            .as_ref()
            .map_or(Path::new("lazin"), |dir| dir.as_path());

        init_directory(directory)?;
        init_default_config(directory)?;

        Ok(())
    }
}
