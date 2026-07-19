use std::path::PathBuf;

use clap::Args;
use lazin_error::LazinResult;

use crate::{
    common::{self},
    exit_error,
    filesystem::link::{DryRunLinker, Linker},
};

#[derive(Args)]
pub(super) struct Link {
    workspace: String,
    #[arg(short = 'd', long = "directory", help = "directory to validate")]
    directory: Option<PathBuf>,
    #[arg(short = 'l', long = "link", help = "actually link")]
    link: bool,
}

impl Link {
    pub(super) fn link(&self) -> LazinResult<()> {
        let config = common::parse_config(self.directory.as_deref())?;
        let workspace_name = &self.workspace;

        if !config.contains_workspace(workspace_name) {
            exit_error!("Workspace '{}' not found", workspace_name)
        }

        if !self.link {
            let mut linker = DryRunLinker::new(config);
            linker.link(workspace_name)?;
        } else {
            #[cfg(unix)]
            {
                use crate::filesystem::link::UnixFSLinker;

                let mut linker = UnixFSLinker::new(config);
                linker.link(workspace_name)?;
            }
        }

        Ok(())
    }
}
