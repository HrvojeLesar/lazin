use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::error::Error;

pub const DEFAULT_DIRECTORY: &str = "lazin";

#[derive(Debug, Deserialize, Hash, PartialEq, Eq)]
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

pub fn files(directory: &Path) -> Result<Vec<TomlFile>, Error> {
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
