use crate::{common::Key, error::Error};

use serde::Deserialize;
use std::{
    collections::HashMap,
    path::{Path, PathBuf},
};

#[derive(Debug, Deserialize)]
pub struct ModuleCompositeValue {
    path: PathBuf,
    #[serde(default)]
    encrypt: bool,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ModuleValue {
    InlinePath(PathBuf),
    CompositeValue(ModuleCompositeValue),
}

impl ModuleValue {
    pub fn is_encrypted(&self) -> bool {
        match self {
            ModuleValue::CompositeValue(module_composite_value) => module_composite_value.encrypt,
            _ => false,
        }
    }

    pub fn path(&self) -> &Path {
        match self {
            ModuleValue::InlinePath(path_buf) => path_buf.as_path(),
            ModuleValue::CompositeValue(module_composite_value) => {
                module_composite_value.path.as_path()
            }
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Module {
    #[serde(flatten)]
    values: HashMap<Key, ModuleValue>,
    #[serde(default)]
    encrypt: bool,
}

impl Module {
    pub fn values_pairs(&self) -> impl Iterator<Item = (&Key, &ModuleValue)> {
        self.values.iter()
    }
}

#[derive(Debug, Deserialize)]
pub struct Modules(HashMap<Key, Module>);

impl Modules {
    pub fn parse(input: &str) -> Result<Self, Error> {
        toml::from_str(input).map_err(|e| e.into())
    }

    pub fn modules(&self) -> impl Iterator<Item = (&Key, &Module)> {
        self.0.iter()
    }
}

#[cfg(test)]
mod test {
    use crate::dotfiles::module::Modules;

    #[test]
    fn parse_module_names() {
        let raw = r#"
        [conf1]
        [conf2]
        "#;

        let result = Modules::parse(raw).expect("raw value should be valid modules config");

        assert!(result.modules().any(|i| i.0.str() == "conf1"));
        assert!(result.modules().any(|i| i.0.str() == "conf2"));
    }

    #[test]
    fn parse_basic_modules() {
        let raw = r#"
        [conf1]
        module1="/some/path"
        module2="/other/path"
        module3="/composite/path"
        "#;

        let result = Modules::parse(raw).expect("raw value should be valid modules config");
        let module = result
            .modules()
            .find(|i| i.0.str() == "conf1")
            .expect("a valid module");

        let module1_path = module
            .1
            .values_pairs()
            .find(|v| v.0.str() == "module1")
            .expect("a valid value pair")
            .1
            .path();

        let module2_path = module
            .1
            .values_pairs()
            .find(|v| v.0.str() == "module2")
            .expect("a valid value pair")
            .1
            .path();

        let module3_path = module
            .1
            .values_pairs()
            .find(|v| v.0.str() == "module3")
            .expect("a valid value pair")
            .1
            .path();

        assert_eq!(module1_path, "/some/path");
        assert_eq!(module2_path, "/other/path");
        assert_eq!(module3_path, "/composite/path");
    }
}
