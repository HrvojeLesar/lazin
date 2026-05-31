use crate::{common, error::Error};

use clap::Args;
use std::path::{Path, PathBuf};

/// Check configuration validity
#[derive(Args)]
pub(super) struct Check {
    #[arg(short = 'd', long = "directory", help = "directory to validate")]
    directory: Option<PathBuf>,
}

impl Check {
    pub(crate) fn check(&self) -> Result<(), Error> {
        let directory = self.directory();
        let files = common::files(directory)?;

        let _workspaces_and_modules = common::parse(&files)?;

        Ok(())
    }

    fn directory(&self) -> &Path {
        common::directory(self.directory.as_deref())
    }
}
