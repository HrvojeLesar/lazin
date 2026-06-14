use std::{
    collections::BTreeMap,
    fmt::Display,
    fs,
    path::{Path, PathBuf},
};

use lazin_pipeline::Bind;

use crate::{
    common::Key,
    dotfiles::{
        config::{Config, RawEntry},
        module::{Module, RawModule},
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
    SourcePathDoesNotExist,
}

impl Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::SourcePathDoesNotExist => write!(f, "source path not found"),
        }
    }
}

type SourceAndModulePairs<'a> = Vec<(&'a Key, &'a RawModule)>;

struct ResolvedConfig;
impl ResolvedConfig {
    pub fn parse(config: &Config) -> LazinResult<Self> {
        let source_and_module_pairs = config
            .entries
            .iter()
            .filter_map(|(source, entry)| match entry {
                RawEntry::Workspace(_) => None,
                RawEntry::Module(raw_module) => Some((source, raw_module)),
            })
            .collect::<SourceAndModulePairs>();

        let validated_module_sources = validate_module_sources(source_and_module_pairs)?;
        let expanded_modules = expand_modules(validated_module_sources)?;

        todo!()
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
                .bind(|path| match !path.exists() {
                    true => Ok(()),
                    false => Err(ValidationError::SourcePathDoesNotExist),
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

fn expand_modules(validated_module_sources: Valid<SourceAndModulePairs>) -> LazinResult<()> {
    let f = validated_module_sources
        .into_inner()
        .into_iter()
        .map(|(module_name, raw_module)| Module::parse(module_name, raw_module))
        .flatten()

    Ok(())
}
