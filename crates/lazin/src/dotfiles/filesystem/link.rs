#[cfg(unix)]
use std::path::Path;
use std::{collections::BTreeMap, fs, path::PathBuf};

use crate::{
    common::Key,
    dotfiles::resolved_config::ResolvedConfig,
    error::{LazinError, LazinResult},
};

enum FileType {
    Link(PathBuf),
    File,
    Directory,
    Override,
    Missing,
}

pub enum PathComparison {
    TargetLinkMissing,
    TargetAndSourceAlreadyLinked,
    TargetIsAnExistingFile,
    TargetIsAnExistingDirectory,
}

pub trait Linker {
    fn link(&mut self, workspace_name: &Key) -> LazinResult<()>;
    fn create_dir_all(&mut self, path: &Path) -> LazinResult<()>;
    fn compare_symlink(&self, source: &Path, target: &Path) -> LazinResult<PathComparison>;
    fn symlink(&mut self, source: &Path, target: &Path) -> LazinResult<()>;
}

struct FSLinker {}

pub struct DryRunLinker {
    config: ResolvedConfig,
    filesystem: BTreeMap<PathBuf, FileType>,
}

impl DryRunLinker {
    pub fn new(config: ResolvedConfig) -> Self {
        Self {
            config,
            filesystem: BTreeMap::default(),
        }
    }
}

impl Linker for DryRunLinker {
    fn link(&mut self, workspace_name: &Key) -> LazinResult<()> {
        let modules = self.config.get_modules_from_workspace_key(workspace_name)?;

        for module in &modules {
            for module_value in &module.values {
                let source = module_value.source.as_path();
                let target = module_value.target.as_path();
                self.create_dir_all(target)?;
                let comparison = self.compare_symlink(source, target)?;
                match comparison {
                    PathComparison::TargetLinkMissing => self.symlink(source, target)?,
                    PathComparison::TargetAndSourceAlreadyLinked => self.symlink(source, target)?,
                    PathComparison::TargetIsAnExistingFile => {
                        lazin_logger::warn!(
                            "Skipping linking {} -> {} - target is an existing file",
                            source.display(),
                            target.display()
                        )
                    }
                    PathComparison::TargetIsAnExistingDirectory => {
                        lazin_logger::warn!(
                            "Skipping linking {} -> {} - target is an existing directory",
                            source.display(),
                            target.display()
                        )
                    }
                }
            }
        }

        Ok(())
    }

    fn create_dir_all(&mut self, mut path: &Path) -> LazinResult<()> {
        if !path.is_dir() {
            return Ok(());
        }

        lazin_logger::debug!("Creating directory: {}", path.display());
        self.filesystem.insert(path.into(), FileType::Directory);
        while path.parent().is_some() {
            path = path
                .parent()
                .ok_or(LazinError::Custom("failed to get path parent"))?;
            self.filesystem.insert(path.into(), FileType::Directory);
        }

        Ok(())
    }

    fn compare_symlink(&self, source: &Path, target: &Path) -> LazinResult<PathComparison> {
        if !target.exists() {
            return Ok(PathComparison::TargetLinkMissing);
        }

        if fs::canonicalize(source)? == fs::canonicalize(target)? {
            todo!("Handle case where source and target are the exact same file");
            return Ok(PathComparison::TargetAndSourceAlreadyLinked);
        }

        if target.is_file() {
            return Ok(PathComparison::TargetIsAnExistingFile);
        }

        if target.is_dir() {
            return Ok(PathComparison::TargetIsAnExistingDirectory);
        }

        unreachable!("Cover all symlink comparison cases dummy");
    }

    fn symlink(&mut self, source: &Path, target: &Path) -> LazinResult<()> {
        lazin_logger::debug!("Linking {} -> {}", source.display(), target.display());
        self.filesystem
            .insert(source.into(), FileType::Link(target.into()));
        Ok(())
    }
}

#[cfg(unix)]
fn copy_permissions(source: &Path, target: &Path) -> LazinResult<()> {
    use std::fs;

    let source_permissions = fs::metadata(source)?.permissions();
    fs::set_permissions(target, source_permissions)?;

    Ok(())
}

fn create_parent_directories(target: &Path) -> LazinResult<()> {
    if let Some(parent_dir) = target.parent() {
        fs::create_dir_all(parent_dir)?;
    }

    Ok(())
}
