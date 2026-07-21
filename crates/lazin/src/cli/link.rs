use std::path::PathBuf;

use clap::Args;
use lazin_error::LazinResult;

use crate::{
    common::{self},
    exit_error,
    filesystem::link::{DryRunLinker, Linker},
};

/// Link selected workspace modules, by default this performs
/// a dry run, use `-l` to link files on disk
#[derive(Args)]
pub(super) struct Link {
    workspace: String,
    #[arg(short = 'd', long = "directory", help = "Directory to validate")]
    directory: Option<PathBuf>,
    #[arg(short = 'l', long = "link", help = "Link selected workspace")]
    link: bool,
    #[arg(
        short = 'f',
        long = "force",
        help = "Force linking, will override non linked files"
    )]
    force: bool,
}

impl Link {
    pub(super) fn link(&self) -> LazinResult<()> {
        let config = common::parse_config(self.directory.as_deref())?;
        let workspace_name = &self.workspace;

        if !config.contains_workspace(workspace_name) {
            exit_error!("Workspace '{}' not found", workspace_name)
        }

        if !self.link {
            let mut linker = DryRunLinker::new(config, self.force);
            linker.link(workspace_name)?;
        } else {
            #[cfg(unix)]
            {
                use crate::filesystem::link::UnixFSLinker;

                let mut linker = UnixFSLinker::new(config, self.force);
                linker.link(workspace_name)?;
            }
        }

        Ok(())
    }
}
