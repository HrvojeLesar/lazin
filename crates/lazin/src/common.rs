use serde::Deserialize;
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{
    diagnostics::toml::TomlDiagnostic,
    dotfiles::{module::Modules, workspace::Workspace},
    error::Error,
};

pub const DEFAULT_DIRECTORY: &str = "lazin";

#[derive(Debug, Clone, Deserialize, Hash, PartialEq, Eq)]
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

pub struct WorkspacesAndModules {
    pub workspaces: Workspace,
    pub modules: Modules,
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

pub fn parse(files: &[TomlFile]) -> Result<WorkspacesAndModules, Error> {
    fn read_file(path: &Path) -> Result<String, Error> {
        fs::read_to_string(path).map_err(Error::from)
    }

    let mut modules = Modules::empty();
    let mut workspaces = Workspace::empty();

    let mut modules_error: Option<Error> = None;
    let mut workspace_error: Option<Error> = None;

    for file in files {
        let data = read_file(&file.path)?;
        match Modules::parse(&data) {
            Ok(other_modules) => modules.join(other_modules)?,
            Err(e) => {
                TomlDiagnostic::new(&file.filename, &data, &e).emit();
                if modules_error.is_none() {
                    modules_error.replace(e);
                }
            }
        }
        match Workspace::parse(&data) {
            Ok(other_workspace) => workspaces.join(other_workspace)?,
            Err(e) => {
                TomlDiagnostic::new(&file.filename, &data, &e).emit();
                if workspace_error.is_none() {
                    workspace_error.replace(e);
                }
            }
        }
    }

    if let Some(error) = modules_error {
        match error {
            Error::TomlParse(_) => return Err(Error::Custom("error in module parsing")),
            _ => return Err(error),
        }
    }

    if let Some(error) = workspace_error {
        match error {
            Error::TomlParse(_) => return Err(Error::Custom("error in workspace parsing")),
            _ => return Err(error),
        }
    }

    Ok(WorkspacesAndModules {
        workspaces,
        modules,
    })
}
