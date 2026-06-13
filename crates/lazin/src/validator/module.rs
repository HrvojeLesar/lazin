use std::{fmt::Display, ops::Deref, path::Path};

use lazin_pipeline::Bind;

use crate::{
    common::Key,
    dotfiles::module::{RawModule, ModuleValue},
};

type ValidationResult = Result<(), Validation>;

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

pub enum ValidationInfo {}

impl Display for ValidationInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "")
    }
}

pub enum Validation {
    Error(ValidationError),
    Info(ValidationInfo),
}

impl Display for Validation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Validation::Error(validation_error) => validation_error.fmt(f),
            Validation::Info(validation_info) => validation_info.fmt(f),
        }
    }
}

impl From<ValidationError> for Validation {
    fn from(value: ValidationError) -> Self {
        Self::Error(value)
    }
}

impl From<ValidationInfo> for Validation {
    fn from(value: ValidationInfo) -> Self {
        Self::Info(value)
    }
}

struct SourcePath<'a>(&'a Path);

impl<'a> Deref for SourcePath<'a> {
    type Target = &'a Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

struct DestinationPath<'a>(&'a Path);

impl<'a> Deref for DestinationPath<'a> {
    type Target = &'a Path;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub struct ModuleValidationError<'a> {
    pub module_name: &'a Key,
    pub module: &'a RawModule,
    pub key: &'a Key,
    pub value: &'a ModuleValue,
    pub validation: Validation,
}

impl Display for ModuleValidationError<'_> {
    fn fmt(&self, _f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        todo!()
    }
}

impl<'a> ModuleValidationError<'a> {
    fn new(
        module_name: &'a Key,
        module: &'a RawModule,
        key: &'a Key,
        value: &'a ModuleValue,
        validation: Validation,
    ) -> Self {
        Self {
            module_name,
            module,
            key,
            value,
            validation,
        }
    }
}

pub struct ModuleValidator;

impl ModuleValidator {
    pub fn validate<'a>(
        module_name: &'a Key,
        module: &'a RawModule,
    ) -> Vec<ModuleValidationError<'a>> {
        let mut errors = Vec::new();
        for (key, value) in module.values_pairs() {
            if let Err(validation_error) = Self::validate_source(key) {
                errors.push(ModuleValidationError::new(
                    module_name,
                    module,
                    key,
                    value,
                    validation_error,
                ));
                continue;
            }
        }

        errors
    }

    fn validate_source(key: &Key) -> Result<(), Validation> {
        let source_path = Path::new(key.str());
        let source_validator = lazin_pipeline::ValidationStep::new(SourcePath(source_path));

        source_validator.bind(Source::exists).result()
    }
}

struct Source;
impl Source {
    fn exists(path: &SourcePath) -> ValidationResult {
        if !path.exists() {
            Err(ValidationError::SourcePathDoesNotExist.into())
        } else {
            Ok(())
        }
    }
}
