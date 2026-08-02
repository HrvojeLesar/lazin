use std::path::PathBuf;

use clap::Args;
use lazin_error::LazinResult;

use crate::{
    common::{self},
    exit_error,
    filesystem::link::{DryRunLinker, Linker, LinkerOptions},
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
    #[arg(
        short = 's',
        long = "skip-failed",
        help = "Skips any files that fail encryption/decryption, by default failure will stop the process partially"
    )]
    skip_failed: bool,
    #[arg(
        short = 'g',
        long = "gitignore",
        help = "Gitignore file, can be a non existing file to not use gitignore at all. By default looks for .gitignore in the current working directory"
    )]
    gitignore: Option<PathBuf>,
}

impl Link {
    pub(super) fn link(&self) -> LazinResult<()> {
        let config = common::parse_config(self.directory.as_deref(), self.gitignore.as_deref())?;
        let workspace_name = &self.workspace;

        if !config.contains_workspace(workspace_name) {
            exit_error!("Workspace '{}' not found", workspace_name)
        }

        let linker_options = LinkerOptions {
            force: self.force,
            should_skip_failed_encryption_decryption: self.skip_failed,
        };

        if !self.link {
            let mut linker = DryRunLinker::new(config, linker_options);
            linker.link(workspace_name)?;
        } else {
            #[cfg(unix)]
            {
                use crate::filesystem::link::UnixFSLinker;

                let mut linker = UnixFSLinker::new(config, linker_options);
                linker.link(workspace_name)?;
            }
        }

        Ok(())
    }
}
