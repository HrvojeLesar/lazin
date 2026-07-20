use lazin_error::{Context, LazinResult};
use std::{
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::exit,
};

use crate::{cache, config, encryption_management, error::LazinError, resolve};

pub const DEFAULT_DIRECTORY: &str = "lazin";

pub struct TomlFile {
    pub path: PathBuf,
    pub filename: String,
}

impl TomlFile {
    pub fn new(path: PathBuf, filename: String) -> Self {
        Self { path, filename }
    }
}

pub fn directory(path: Option<&Path>) -> &Path {
    path.unwrap_or(Path::new(DEFAULT_DIRECTORY))
}

pub fn files(directory: &Path) -> LazinResult<Vec<TomlFile>> {
    let mut files = Vec::new();

    for entry in fs::read_dir(directory).context("Failed to read directory")? {
        let entry = entry.context("Failed to read directory entry")?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "toml")
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
        {
            files.push(TomlFile::new(path.clone(), name.to_string()));
        }
    }

    Ok(files)
}

pub fn parse_config(config_directory: Option<&Path>) -> LazinResult<resolve::config::Config> {
    let directory = directory(config_directory);

    match directory.try_exists() {
        Ok(true) => {}
        Ok(false) => return Err(LazinError::directory_does_not_exist(directory).into()),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Err(LazinError::directory_does_not_exist(directory).into());
        }
        Err(e) => return Err(LazinError::Io(e)).context("Failed to check if directory exists"),
    }

    let config = config::Config::parse(directory)?;

    let cache = cache::Cache::try_new(directory)?;
    // TODO: add configurable gitignore location, current directory should be default
    // or walk down to a closes directory with .git directory
    let gitignore = encryption_management::gitignore::Gitignore::load("./.gitignore")?;
    let encryption_manager = encryption_management::EncryptionManager::new(cache);

    resolve::config::Config::parse(config, encryption_manager, gitignore)
}

#[inline]
pub fn exit_success() -> ! {
    exit(0)
}

#[macro_export]
macro_rules! exit_error {
    () => {{
        std::process::exit(1)
    }};
    ($arg:expr) => {{
        ::lazin_logger::error!($arg);
        std::process::exit(1)
    }};
    ($($arg:tt)+) => {{
        ::lazin_logger::error!($($arg)+);
        std::process::exit(1)
    }};
}

#[inline]
#[allow(unused)]
pub fn exit_error_with_code(code: i32) -> ! {
    exit(code)
}
