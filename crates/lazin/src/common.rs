use lazin_error::{Context, LazinResult};
use serde::Deserialize;
use std::{
    fmt::Display,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::exit,
};

use crate::{cache, config::config, encryption_management, error::LazinError, resolve};

pub const DEFAULT_DIRECTORY: &str = "lazin";

#[derive(Debug, Clone, Deserialize, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Key(String);

impl Key {
    pub fn str(&self) -> &str {
        self.0.as_str()
    }
}

impl From<String> for Key {
    fn from(value: String) -> Self {
        Self(value)
    }
}

impl Display for Key {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

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

    // let config = Config::parse(directory)?;
    let config = config::Config::parse(directory)?;

    let cache = cache::Cache::try_new(directory)?;
    let gitignore = encryption_management::gitignore::Gitignore::load(".")?;
    let encryption_manager = encryption_management::EncryptionManager::new(cache, gitignore);

    resolve::config::Config::parse(config, encryption_manager)
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

pub fn check(config_directory: Option<&Path>) -> LazinResult<resolve::config::Config> {
    let config = parse_config(config_directory)?;

    Ok(config)
}
