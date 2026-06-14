use std::path::PathBuf;

use clap::Args;

use crate::{
    common::{self, Key},
    error::LazinResult,
    exit_error,
};

#[derive(Args)]
pub(super) struct Link {
    workspace: String,
    #[arg(short = 'd', long = "directory", help = "directory to validate")]
    directory: Option<PathBuf>,
}

impl Link {
    pub(super) fn link(&self) -> LazinResult<()> {
        let config = common::check(self.directory.as_deref())?;
        let workspace_name = Key::from(self.workspace.clone());

        if !config.workspaces.contains_key(&workspace_name) {
            exit_error!("Workspace '{}' not found", workspace_name)
        }

        todo!("workspace logic")
    }
}
