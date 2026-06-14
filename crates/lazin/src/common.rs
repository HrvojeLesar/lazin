use serde::Deserialize;
use std::{
    fmt::Display,
    fs,
    io::ErrorKind,
    path::{Path, PathBuf},
    process::exit,
};

use crate::{
    dotfiles::config::Config,
    error::{Error, LazinResult},
    validator::module::{ModuleValidationError, ModuleValidator},
};

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

    for entry in fs::read_dir(directory).map_err(Error::from)? {
        let entry = entry.map_err(Error::from)?;
        let path = entry.path();

        if path.extension().is_some_and(|ext| ext == "toml")
            && let Some(name) = path.file_name().and_then(|n| n.to_str())
        {
            files.push(TomlFile::new(path.clone(), name.to_string()));
        }
    }

    Ok(files)
}

pub fn parse_config(config_directory: Option<&Path>) -> LazinResult<Config> {
    let directory = directory(config_directory);

    match directory.try_exists() {
        Ok(true) => {}
        Ok(false) => return Err(Error::directory_does_not_exist(directory)),
        Err(e) if e.kind() == ErrorKind::NotFound => {
            return Err(Error::directory_does_not_exist(directory));
        }
        Err(e) => return Err(e.into()),
    }

    Config::parse(directory)
}

pub fn validate_config(config: &Config) -> Vec<ModuleValidationError<'_>> {
    let modules = config.modules();
    let mut validation_errors = Vec::new();
    for (module_name, module) in modules {
        validation_errors.extend(ModuleValidator::validate(module_name, module));
    }

    validation_errors
}

pub fn report_validation_errors(mut errors: Vec<ModuleValidationError>) {
    errors.sort_by(|validation_a, validation_b| validation_a.key.cmp(validation_b.key));
    errors.iter().for_each(|err| {
        lazin_logger::error!(
            "Module '{}' path '{}': {}",
            err.module_name.str(),
            err.key.str(),
            err.validation
        );
    });
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
pub fn check(config_directory: Option<&Path>) -> LazinResult<Config> {
    let config = parse_config(config_directory)?;

    let validation_errors = validate_config(&config);

    if !validation_errors.is_empty() {
        report_validation_errors(validation_errors);
        exit_error!()
    }

    Ok(config)
}
