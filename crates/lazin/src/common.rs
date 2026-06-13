use serde::Deserialize;
use std::{
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

    // TODO: maybe make this block nicer
    match directory.try_exists() {
        Ok(exists) => {
            if !exists {
                return Err(Error::directory_does_not_exist(directory));
            }
        }
        Err(e) => match e.kind() {
            ErrorKind::NotFound => return Err(Error::directory_does_not_exist(directory)),
            _ => return Err(e.into()),
        },
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

#[inline]
pub fn exit_error() -> ! {
    exit(1)
}

#[inline]
#[allow(unused)]
pub fn exit_error_with_code(code: i32) -> ! {
    exit(code)
}
