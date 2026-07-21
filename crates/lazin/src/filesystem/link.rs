use std::cell::RefCell;
use std::path::Path;
use std::{collections::BTreeMap, fs, path::PathBuf};

use lazin_error::{Context, LazinResult};

use crate::encryption_management::EncryptionManager;
use crate::error::LazinError;
use crate::resolve;

#[allow(unused)]
enum FileType {
    Link,
    Directory,
    File,
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
    fn link(&mut self, workspace_name: &str) -> LazinResult<()>;
    fn create_dir_all(&self, path: &Path) -> LazinResult<()>;

    // TODO: Fix failures on invalid symlinks
    fn compare_symlink(&self, source: &Path, target: &Path) -> LazinResult<PathComparison> {
        match target.try_exists() {
            Ok(true) => (),
            Ok(false) => return Ok(PathComparison::TargetLinkMissing),
            Err(e) => return Err(LazinError::Io(e)).context("Failed to check if target exists"),
        }

        let canonicalized_source =
            fs::canonicalize(source).context("Failed to canonicalize source")?;
        let canonicalized_target =
            fs::canonicalize(target).context("Failed to canonicalize target")?;

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
    fn symlink(&self, source: &Path, target: &Path) -> LazinResult<()>;
}

#[cfg(unix)]
pub struct UnixFSLinker {
    config: resolve::config::Config,
    force: bool,
}
impl UnixFSLinker {
    pub(crate) fn new(config: resolve::config::Config, force: bool) -> Self {
        Self { config, force }
    }
}

#[cfg(unix)]
impl Linker for UnixFSLinker {
    fn link(&mut self, workspace_name: &str) -> LazinResult<()> {
        let modules = self.config.get_workspace_modules(workspace_name);
        let encrypt_options = LinkOptions {
            output_override_path: None,
            encryption_manager: &self.config.encryption_manager,
            force: self.force,
        };

        link(self, &modules, encrypt_options)?;

        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> LazinResult<()> {
        create_parent_directories(path)
    }

    fn symlink(&self, source: &Path, target: &Path) -> LazinResult<()> {
        use std::os::unix::fs::symlink;

        let abolute_source =
            fs::canonicalize(source).context("Failed to get absolute path for source")?;

        lazin_logger::info!(
            "Linking {} -> {}",
            abolute_source.display(),
            target.display()
        );

        // TODO: add sudo linking, config option to prompt for sudo when linking
        // a file which requires elevated permissions
        if target.exists() {
            match fs::remove_file(target) {
                Ok(_) => {}
                Err(e) => match e.kind() {
                    std::io::ErrorKind::PermissionDenied => {
                        lazin_logger::warn!(
                            "Failed to remove existing file at target: '{}', this file will be skipped",
                            target.display()
                        );

                        return Ok(());
                    }
                    _ => Err(e).context("Failed to remove file before linking")?,
                },
            }
        }

        symlink(&abolute_source, target).context("Failed to symlink")?;
        copy_permissions(&abolute_source, target)?;

        Ok(())
    }
}

pub struct DryRunLinker {
    config: resolve::config::Config,
    filesystem: RefCell<BTreeMap<PathBuf, FileType>>,
    force: bool,
}

impl DryRunLinker {
    pub fn new(config: resolve::config::Config, force: bool) -> Self {
        Self {
            config,
            filesystem: RefCell::default(),
            force,
        }
    }
}

impl Linker for DryRunLinker {
    fn link(&mut self, workspace_name: &str) -> LazinResult<()> {
        let modules = self.config.get_workspace_modules(workspace_name);
        let encrypt_options = LinkOptions {
            output_override_path: Some(Path::new("/dev/null")),
            encryption_manager: &self.config.encryption_manager,
            force: self.force,
        };

        link(self, &modules, encrypt_options)?;

        Ok(())
    }

    fn create_dir_all(&self, mut path: &Path) -> LazinResult<()> {
        if !path.is_dir() {
            return Ok(());
        }

        lazin_logger::info!("Creating directory: {}", path.display());
        self.filesystem
            .borrow_mut()
            .insert(path.into(), FileType::Directory);
        while path.parent().is_some() {
            path = path
                .parent()
                .ok_or(LazinError::Custom("failed to get path parent"))?;
            self.filesystem
                .borrow_mut()
                .insert(path.into(), FileType::Directory);
        }

        Ok(())
    }

    fn symlink(&self, source: &Path, target: &Path) -> LazinResult<()> {
        lazin_logger::info!("Linking {} -> {}", source.display(), target.display());
        self.filesystem
            .borrow_mut()
            .insert(source.into(), FileType::Link);
        Ok(())
    }
}

struct LinkOptions<'a> {
    output_override_path: Option<&'a Path>,
    encryption_manager: &'a EncryptionManager,
    force: bool,
}

fn link<T: Linker>(
    linker: &T,
    modules: &Vec<&resolve::module::Module>,
    options: LinkOptions<'_>,
) -> LazinResult<()> {
    for module in modules {
        for module_value in &module.values {
            let source = &module_value.source;
            let target = &module_value.target;
            linker.create_dir_all(target)?;
            match (linker.compare_symlink(source, target)?, options.force) {
                (PathComparison::TargetLinkMissing, _)
                | (PathComparison::TargetAndSourceAlreadyLinked, true)
                | (PathComparison::TargetIsAnExistingFile, true) => {
                    match module_value.encryption {
                        resolve::module::Encryption::Disabled => {}
                        resolve::module::Encryption::Enabled { .. } => {
                            let decryption_source_file =
                                EncryptionManager::get_input_file_with_extension(source);
                            let decryption_output_file =
                                options.output_override_path.unwrap_or(target);
                            lazin_logger::info!(
                                "Decrypting file {} into {}",
                                decryption_source_file.display(),
                                decryption_output_file.display()
                            );

                            options
                                .encryption_manager
                                .manage_decryption(source, options.output_override_path)?
                        }
                    }

                    linker.symlink(source, target)?
                }
                (PathComparison::TargetAndSourceAlreadyLinked, false) => {
                    lazin_logger::warn!(
                        "Skipping linking {} -> {} - target is already linked",
                        source.display(),
                        target.display()
                    )
                }
                (PathComparison::TargetIsAnExistingFile, false) => {
                    lazin_logger::error!(
                        "Skipping linking {} -> {} - target is an existing file",
                        source.display(),
                        target.display()
                    )
                }
                (PathComparison::TargetIsAnExistingDirectory, _) => {
                    lazin_logger::error!(
                        "Skipping linking {} -> {} - target is an existing directory",
                        source.display(),
                        target.display()
                    )
                }
                (PathComparison::Unknown, _) => {
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

#[cfg(unix)]
fn copy_permissions(source: &Path, target: &Path) -> LazinResult<()> {
    use std::fs;

    let source_permissions = fs::metadata(source)
        .context("Failed to get source metadata")?
        .permissions();
    fs::set_permissions(target, source_permissions).context("Failed to set permissions")?;

    Ok(())
}

fn create_parent_directories(target: &Path) -> LazinResult<()> {
    if let Some(parent_dir) = target.parent()
        && !parent_dir.exists()
    {
        fs::create_dir_all(parent_dir).context("Failed to create directories")?;
        lazin_logger::info!("Creating directory: {}", parent_dir.display());
    }

    Ok(())
}
