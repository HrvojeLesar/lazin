use std::{
    collections::BTreeMap,
    fmt::Display,
    path::{Path, PathBuf},
};

use lazin_pipeline::Bind;

use crate::{
    common::Key,
    dotfiles::{
        config::{Config, RawEntry},
        module::{Module, RawModule},
        workspace::Workspace,
    },
    error::{LazinError, LazinResult},
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

type SourceAndModulePairs<'a> = Vec<(&'a Key, &'a RawModule)>;

#[derive(Debug)]
pub struct ResolvedConfig {
    pub modules: BTreeMap<Key, Module>,
    pub workspaces: BTreeMap<Key, Workspace>,
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
            .collect::<SourceAndModulePairs>();

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
            .map(|workspace| {
                workspace.modules.iter().try_for_each(|module_name| {
                    match modules.contains_key(module_name) {
                        true => Ok(()),
                        false => Err(LazinError::ModuleNotFound(module_name.str().to_string())),
                    }
                })?;
                Ok((workspace.name.clone(), workspace))
            })
            .collect::<LazinResult<BTreeMap<Key, Workspace>>>()?;

        Ok(Self {
            modules,
            workspaces,
        })
    }

    pub fn get_modules_from_workspace_key(&self, workspace_key: &Key) -> LazinResult<Vec<Module>> {
        let workspace = self
            .workspaces
            .get(workspace_key)
            .ok_or(LazinError::WorkspaceNotFound(workspace_key.clone()))?;

        Ok(self
            .modules
            .iter()
            .filter_map(|(k, m)| match workspace.modules.contains(k) {
                true => Some(m.clone()),
                false => None,
            })
            .collect())
    }
}

fn validate_module_sources(
    pairs: SourceAndModulePairs,
) -> LazinResult<Valid<SourceAndModulePairs>> {
    let valid = true;
    match pairs
        .iter()
        .map(|pair| {
            let source_path = Path::new(pair.0.str());
            lazin_pipeline::ValidationStep::new(source_path)
                .bind(|path| match path.exists() {
                    true => Ok(()),
                    false => Err(ValidationError::SourcePathDoesNotExist {
                        module_name: pair.0.clone(),
                        path: PathBuf::from(path),
                    }),
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
        true => Ok(Valid(pairs)),
        false => Err(LazinError::InvalidModuleSources),
    }
}

fn expand_modules(
    validated_module_sources: Valid<SourceAndModulePairs>,
) -> LazinResult<BTreeMap<Key, Module>> {
    validated_module_sources
        .into_inner()
        .into_iter()
        .map(|(module_name, raw_module)| {
            let module = Module::parse(module_name, raw_module)?;
            Ok((module.name.clone(), module))
        })
        .collect()
}
