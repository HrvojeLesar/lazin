use std::collections::{HashMap, hash_map};

use serde::Deserialize;

use crate::{
    common::Key,
    dotfiles::{module::Module, workspace::Workspace},
    error::{DuplicateKeysError, Error},
};

#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(flatten)]
    entries: HashMap<Key, Entry>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Entry {
    Workspace(Workspace),
    Module(Module),
}

impl Config {
    pub fn parse(input: &str) -> Result<Self, Error> {
        toml::from_str(input).map_err(Error::from)
    }

    pub fn empty() -> Self {
        Self {
            entries: HashMap::default(),
        }
    }

    pub fn join(&mut self, other: Self) -> Result<(), DuplicateKeysError> {
        let mut duplicate_keys_errors = Vec::new();

        for (key, module) in other.entries {
            match self.entries.entry(key) {
                hash_map::Entry::Occupied(entry) => {
                    duplicate_keys_errors.push(entry.key().clone());
                }
                hash_map::Entry::Vacant(entry) => {
                    entry.insert(module);
                }
            }
        }

        if !duplicate_keys_errors.is_empty() {
            return Err(DuplicateKeysError {
                duplicates: duplicate_keys_errors,
            });
        }

        Ok(())
    }

    pub fn workspaces(&self) -> Vec<(&Key, &Workspace)> {
        self.entries
            .iter()
            .filter_map(|entry| match entry.1 {
                Entry::Workspace(w) => Some((entry.0, w)),
                Entry::Module(_) => None,
            })
            .collect()
    }

    pub fn modules(&self) -> Vec<(&Key, &Module)> {
        self.entries
            .iter()
            .filter_map(|entry| match entry.1 {
                Entry::Workspace(_) => None,
                Entry::Module(m) => Some((entry.0, m)),
            })
            .collect()
    }
}

#[cfg(test)]
mod test {
    use super::{Config, Entry};
    use crate::common::Key;

    fn entry<'a>(config: &'a Config, name: &str) -> &'a Entry {
        config
            .entries
            .iter()
            .find(|(key, _)| key.str() == name)
            .map(|(_, entry)| entry)
            .unwrap_or_else(|| panic!("no entry named `{name}`"))
    }

    #[test]
    fn parses_workspace_members() {
        let config = Config::parse(r#"workspace1 = ["module1", "module2"]"#)
            .expect("a valid workspace config");

        match entry(&config, "workspace1") {
            Entry::Workspace(workspace) => {
                let members: Vec<&str> = workspace.modules().iter().map(Key::str).collect();
                assert_eq!(members, ["module1", "module2"]);
            }
            Entry::Module(_) => panic!("`workspace1` should be classified as a workspace"),
        }
    }

    #[test]
    fn parses_empty_workspace() {
        let config = Config::parse("workspace1 = []").expect("a valid empty workspace");

        match entry(&config, "workspace1") {
            Entry::Workspace(workspace) => assert!(workspace.modules().is_empty()),
            Entry::Module(_) => panic!("an empty array should be classified as a workspace"),
        }
    }

    #[test]
    fn parses_multiple_workspaces() {
        let config = Config::parse(
            r#"
            workspace1 = ["module1"]
            workspace2 = ["module2", "module3"]
            "#,
        )
        .expect("valid workspaces");

        assert!(matches!(entry(&config, "workspace1"), Entry::Workspace(_)));
        assert!(matches!(entry(&config, "workspace2"), Entry::Workspace(_)));
    }

    #[test]
    fn distinguishes_workspaces_from_modules() {
        let config = Config::parse(
            r#"
            workspace1 = ["module1"]

            [module1]
            file = "/some/path"
            "#,
        )
        .expect("a valid mixed config");

        assert!(matches!(entry(&config, "workspace1"), Entry::Workspace(_)));
        assert!(matches!(entry(&config, "module1"), Entry::Module(_)));
    }

    #[test]
    fn rejects_workspace_with_non_string_members() {
        let result = Config::parse("workspace1 = [1, 2, 3]");

        assert!(
            result.is_err(),
            "numeric array members must not parse as a workspace"
        );
    }
}
