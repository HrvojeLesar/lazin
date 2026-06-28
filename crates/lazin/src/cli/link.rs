use std::path::PathBuf;

use clap::Args;
use lazin_error::LazinResult;

use crate::{
    common::{self, Key},
    dotfiles::filesystem::link::{DryRunLinker, Linker},
    exit_error,
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
        let config = common::check(self.directory.as_deref())?;
        let workspace_name = Key::from(self.workspace.clone());

        if !config.workspaces.contains_key(&workspace_name) {
            exit_error!("Workspace '{}' not found", workspace_name)
        }

        if !self.link {
            let mut linker = DryRunLinker::new(config);
            linker.link(&workspace_name)?;
        } else {
            #[cfg(unix)]
            {
                use crate::dotfiles::filesystem::link::UnixFSLinker;

                let mut linker = UnixFSLinker::new(config);
                linker.link(&workspace_name)?;
            }
        }

        Ok(())
    }
}
