use std::{
    collections::BTreeMap,
    fmt::Display,
    path::{Path, PathBuf},
};

use lazin_error::LazinResult;
use lazin_pipeline::Bind;

use crate::{
    common::Key,
    dotfiles::{
        config::{Config, RawEntry},
        module::{Module, ModuleName, config::ModuleConfig},
        workspace::{Workspace, WorkspaceName},
    },
    error::Error,
};

pub struct Valid<T>(T);
impl<T> Valid<T> {
    pub fn into_inner(self) -> T {
        self.0
    }
}

pub enum ValidationError {
    SourcePathDoesNotExist { module_name: Key, path: PathBuf },
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::SourcePathDoesNotExist { module_name, path } => {
                write!(
                    f,
                    "Module '{}' has a source path that could not be found: '{}'",
                    module_name,
                    path.display()
                )
            }
        }
    }
}

type ModuleNameAndRawModule<'a> = Vec<(&'a Key, &'a ModuleConfig)>;

#[derive(Debug)]
pub struct ResolvedConfig {
    pub modules: BTreeMap<ModuleName, Module>,
    pub workspaces: BTreeMap<WorkspaceName, Workspace>,
}

impl ResolvedConfig {
    pub fn parse(config: Config) -> LazinResult<Self> {
        let source_and_module_pairs = config
            .entries
            .iter()
            .filter_map(|(source, entry)| match entry {
                RawEntry::Workspace(_) => None,
                RawEntry::Module(raw_module) => Some((source, raw_module)),
            })
            .collect::<ModuleNameAndRawModule>();

        let validated_module_sources = validate_module_sources(source_and_module_pairs)?;
        let modules = expand_modules(validated_module_sources)?;

        let workspaces = config
            .entries
            .into_iter()
            .filter_map(|(source, entry)| match entry {
                RawEntry::Workspace(raw_workspace) => {
                    Some(Workspace::new(source.clone(), raw_workspace.modules))
                }
                RawEntry::Module(_) => None,
            })
            .map(|workspace| -> LazinResult<(WorkspaceName, Workspace)> {
                workspace.modules.iter().try_for_each(|module_name| {
                    match modules.contains_key(module_name) {
                        true => Ok(()),
                        false => Err(Error::ModuleNotFound(
                            workspace.name.clone(),
                            ModuleName(module_name.clone()),
                        )),
                    }
                })?;
                Ok((workspace.name.clone(), workspace))
            })
            .collect::<LazinResult<BTreeMap<WorkspaceName, Workspace>>>()?;

        Ok(Self {
            modules,
            workspaces,
        })
    }

    pub fn get_modules_from_workspace_key(&self, workspace_key: &Key) -> LazinResult<Vec<Module>> {
        let workspace = self
            .workspaces
            .get(workspace_key)
            .ok_or(Error::WorkspaceNotFound(workspace_key.clone()))?;

        Ok(self
            .modules
            .iter()
            .filter_map(|(k, m)| match workspace.modules.contains(k.as_ref()) {
                true => Some(m.clone()),
                false => None,
            })
            .collect())
    }
}

fn validate_module_sources(
    pairs: ModuleNameAndRawModule,
) -> LazinResult<Valid<ModuleNameAndRawModule>> {
    let valid = true;
    pairs.iter().try_for_each(|pair| {
        match pair
            .1
            .values
            .keys()
            .map(|source_path| {
                lazin_pipeline::new(source_path)
                    .bind(|path| {
                        let path = Path::new(path.0.str());
                        match path.exists() {
                            true => Ok(()),
                            false => Err(ValidationError::SourcePathDoesNotExist {
                                module_name: pair.0.clone(),
                                path: path.into(),
                            }),
                        }
                    })
                    .result()
            })
            .fold(valid, |acc, result| match result {
                Ok(_) => acc,
                Err(e) => {
                    lazin_logger::error!(e);
                    false
                }
            }) {
            true => Ok(()),
            false => Err(Error::InvalidModuleSources),
        }
    })?;

    Ok(Valid(pairs))
}

fn expand_modules(
    validated_module_sources: Valid<ModuleNameAndRawModule>,
) -> LazinResult<BTreeMap<ModuleName, Module>> {
    validated_module_sources
        .into_inner()
        .into_iter()
        .map(|(module_name, raw_module)| {
            let module = Module::parse(module_name, raw_module)?;
            Ok((module.name.clone(), module))
        })
        .collect()
}
