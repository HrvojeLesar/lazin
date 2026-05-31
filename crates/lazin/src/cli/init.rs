use std::path::{Path, PathBuf};

use crate::dotfiles::filesystem::init::{init_default_config, init_directory};

use super::error::Error;
use clap::Args;

/// Initializes new `lazin` example configuration
///
/// Creates a directory with example configuration files.
#[derive(Args)]
pub(super) struct Init {
    #[arg(
        short = 'd',
        long = "directory",
        help = "directory to write the example configuration in"
    )]
    directory: Option<PathBuf>,
}

impl Init {
    const DEFAULT_DIRECTORY: &str = "lazin";

    pub(crate) fn init(&self) -> Result<(), Error> {
        let directory = self.directory();

        init_directory(directory)?;
        init_default_config(directory)?;

        Ok(())
    }

    fn directory(&self) -> &Path {
        self.directory
            .as_deref()
            .unwrap_or(Path::new(Self::DEFAULT_DIRECTORY))
    }
}

#[cfg(test)]
mod test {
    use std::path::{Path, PathBuf};

    use crate::{cli::init::Init, test::filesystem::tmp::TempDir};

    #[test]
    fn defaults_to_lazin_directory() {
        let init = Init { directory: None };
        assert_eq!(init.directory(), Path::new(Init::DEFAULT_DIRECTORY));
    }

    #[test]
    fn uses_supplied_directory() {
        let init = Init {
            directory: Some(PathBuf::from("/some/path")),
        };
        assert_eq!(init.directory(), Path::new("/some/path"));
    }

    #[test]
    fn init_writes_config_into_directory() {
        let temp = TempDir::new();
        let init = Init {
            directory: Some(temp.path().to_path_buf()),
        };

        init.init().expect("init succeeds");

        assert!(temp.path().exists());
        assert!(temp.path().join("workspace.toml").exists());
        assert!(temp.path().join("modules.toml").exists());
    }
}
