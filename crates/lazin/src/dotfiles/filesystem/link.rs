use std::io::ErrorKind;
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
    Unknown,
}

pub trait Linker {
    fn link(&mut self, workspace_name: &Key) -> LazinResult<()>;
    fn create_dir_all(&mut self, path: &Path) -> LazinResult<()>;

    // TODO: Fix failures on invalid symlinks
    fn compare_symlink(&self, source: &Path, target: &Path) -> LazinResult<PathComparison> {
        match target.try_exists() {
            Ok(true) => (),
            Ok(false) => return Ok(PathComparison::TargetLinkMissing),
            Err(e) => return Err(LazinError::IoExt("Failed to check if target exists", e)),
        }

        let canonicalized_source = fs::canonicalize(source)
            .map_err(|e| LazinError::IoExt("Failed to canonicalize source", e))?;
        let canonicalized_target = fs::canonicalize(target)
            .map_err(|e| LazinError::IoExt("Failed to canonicalize target", e))?;

        //TODO:  Handle case where source and target are the exact same file
        if canonicalized_source == canonicalized_target {
            return Ok(PathComparison::TargetAndSourceAlreadyLinked);
        }

        if target.is_file() {
            return Ok(PathComparison::TargetIsAnExistingFile);
        }

        if target.is_dir() {
            return Ok(PathComparison::TargetIsAnExistingDirectory);
        }

        Ok(PathComparison::Unknown)
    }
    fn symlink(&mut self, source: &Path, target: &Path) -> LazinResult<()>;
}

#[cfg(unix)]
pub struct UnixFSLinker {
    config: ResolvedConfig,
}
impl UnixFSLinker {
    pub(crate) fn new(config: ResolvedConfig) -> Self {
        Self { config }
    }
}

#[cfg(unix)]
impl Linker for UnixFSLinker {
    // TODO: duplicate from dry run linker
    fn link(&mut self, workspace_name: &Key) -> LazinResult<()> {
        let modules = self.config.get_modules_from_workspace_key(workspace_name)?;

        for module in &modules {
            for module_value in &module.values {
                let source = module_value.source.as_path();
                let target = module_value.target.as_path();
                self.create_dir_all(target)?;
                match self.compare_symlink(source, target)? {
                    PathComparison::TargetLinkMissing => self.symlink(source, target)?,
                    PathComparison::TargetAndSourceAlreadyLinked => {
                        lazin_logger::warn!(
                            "Skipping linking {} -> {} - target is already linked",
                            source.display(),
                            target.display()
                        )
                    }
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
                    PathComparison::Unknown => {
                        lazin_logger::error!(
                            "Skipping linking {} -> {} - unknown path comparison; this is a bug and this case should be handled",
                            source.display(),
                            target.display()
                        )
                    }
                }
            }
        }

        Ok(())
    }

    fn create_dir_all(&mut self, path: &Path) -> LazinResult<()> {
        create_parent_directories(path)
    }

    fn symlink(&mut self, source: &Path, target: &Path) -> LazinResult<()> {
        use std::os::unix::fs::symlink;

        let abolute_source = fs::canonicalize(source)
            .map_err(|e| LazinError::IoExt("failed to get absolute path for source", e))?;

        lazin_logger::info!(
            "Linking {} -> {}",
            abolute_source.display(),
            target.display()
        );
        symlink(abolute_source, target).map_err(|e| LazinError::IoExt("Failed to symlink", e))?;
        copy_permissions(source, target)?;

        Ok(())
    }
}

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
    // TODO: duplicate
    fn link(&mut self, workspace_name: &Key) -> LazinResult<()> {
        let modules = self.config.get_modules_from_workspace_key(workspace_name)?;

        for module in &modules {
            for module_value in &module.values {
                let source = module_value.source.as_path();
                let target = module_value.target.as_path();
                self.create_dir_all(target)?;
                match self.compare_symlink(source, target)? {
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
                    PathComparison::Unknown => {
                        lazin_logger::error!(
                            "Skipping linking {} -> {} - unknown path comparison; this is a bug and this case should be handled",
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

    let source_permissions = fs::metadata(source)
        .map_err(|e| LazinError::IoExt("Faild to get source metadata", e))?
        .permissions();
    fs::set_permissions(target, source_permissions)
        .map_err(|e| LazinError::IoExt("Failed to set permissions", e))?;

    Ok(())
}

fn create_parent_directories(target: &Path) -> LazinResult<()> {
    if let Some(parent_dir) = target.parent()
        && !parent_dir.exists()
    {
        fs::create_dir_all(parent_dir)
            .map_err(|e| LazinError::IoExt("Failed to create directories", e))?;
        lazin_logger::info!("Creating directory: {}", parent_dir.display());
    }

    Ok(())
}
