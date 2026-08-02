use lazin_error::LazinResult;

use crate::common;
use clap::Args;
use std::path::PathBuf;

/// Lists configured workspaces
#[derive(Args)]
pub(super) struct Workspaces {
    #[arg(
        short = 'd',
        long = "directory",
        help = "Lists workspaces configured in provided directory"
    )]
    directory: Option<PathBuf>,
    #[arg(
        short = 'm',
        long = "modules",
        help = "Include a list of configured modules for the workspace"
    )]
    include_modules: bool,
    #[arg(
        short = 'g',
        long = "gitignore",
        help = "Gitignore file, can be a non existing file to not use gitignore at all. By default looks for .gitignore in the current working directory"
    )]
    gitignore: Option<PathBuf>,
}

impl Workspaces {
    pub fn list_workspaces(&self) -> LazinResult<()> {
        let config = common::parse_config(self.directory.as_deref(), self.gitignore.as_deref())?;

        if config.workspaces.is_empty() {
            lazin_logger::warn!("No workspaces configured");
            return Ok(());
        }

        lazin_logger::info!("Configured workspaces:");
        for (idx, workspace) in config.workspaces.iter().enumerate() {
            let enumerator = idx + 1;
            lazin_logger::print!("{}. {}", enumerator, &workspace.name);
            if self.include_modules {
                let modules = config.get_workspace_modules(&workspace.name);
                for module in modules {
                    lazin_logger::print!("\t{}", module.name);
                }
            }
        }

        Ok(())
    }
}
