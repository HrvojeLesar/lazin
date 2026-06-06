use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    diagnostics::toml::TomlDiagnostic,
    dotfiles::config::Config,
    error::{Error, LazinResult},
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

pub fn parse(files: &[TomlFile]) -> LazinResult<Config> {
    fn read_file(path: &Path) -> LazinResult<String> {
        fs::read_to_string(path).map_err(Error::from)
    }

    let mut config = Config::empty();
    let mut error = None;

    for file in files {
        let data = read_file(&file.path)?;
        match Config::parse(&data) {
            Ok(other) => config.join(other)?,
            Err(e) => {
                TomlDiagnostic::new(&file.filename, &data, &e).emit();
                if error.is_none() {
                    error.replace(e);
                }
            }
        }
    }

    if let Some(error) = error {
        match error {
            Error::TomlParse(_) => return Err(Error::Custom("error in config parsing")),
            _ => return Err(error),
        }
    }

    Ok(config)
}

pub fn parse_config(config_directory: Option<&Path>) -> LazinResult<Config> {
    let directory = directory(config_directory);
    let files = files(directory)?;

    parse(&files)
}
