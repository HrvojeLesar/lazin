use std::cell::RefCell;
use std::io::ErrorKind;
use std::path::Path;
use std::{collections::BTreeMap, fs, path::PathBuf};

use lazin_error::{Context, LazinResult};

use crate::encryption_management::EncryptionManager;
use crate::error::LazinError;
use crate::filesystem::privileged::ElevationCheck;
#[cfg(unix)]
use crate::filesystem::privileged::Elevator;
use crate::resolve;

#[allow(unused)]
enum FileType {
    Link,
    Directory,
    File,
    Override,
    Missing,
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ElevationPolicy {
    #[default]
    Prompt,
    Always,
    Never,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Performed {
    Yes,
    Skipped,
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
    fn create_dir_all(&self, path: &Path) -> LazinResult<Performed>;

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

pub struct LinkerOptions {
    pub force: bool,
    pub should_skip_failed_encryption_decryption: bool,
    pub elevation_policy: ElevationPolicy,
}

#[cfg(unix)]
pub struct UnixFSLinker {
    config: resolve::config::Config,
    force: bool,
    should_skip_failed_encryption_decryption: bool,
    elevator: Elevator,
}

impl UnixFSLinker {
    pub(crate) fn new(config: resolve::config::Config, linker_options: LinkerOptions) -> Self {
        Self {
            config,
            force: linker_options.force,
            should_skip_failed_encryption_decryption: linker_options
                .should_skip_failed_encryption_decryption,
            elevator: Elevator::new(linker_options.elevation_policy),
        }
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
            should_skip_failed_encryption_decryption: self.should_skip_failed_encryption_decryption,
        };

        link(self, &modules, encrypt_options)?;

        Ok(())
    }

    fn create_dir_all(&self, path: &Path) -> LazinResult<Performed> {
        let Some(parent_directory) = missing_parent_directory(path) else {
            return Ok(Performed::Yes);
        };

        match fs::create_dir_all(parent_directory) {
            Ok(()) => {
                lazin_logger::info!("Creating directory: {}", parent_directory.display());

                Ok(Performed::Yes)
            }
            Err(e) if e.kind() == ErrorKind::PermissionDenied => {
                self.elevator.create_dir_all(parent_directory)
            }
            Err(e) => Err(e).context("Failed to create directories"),
        }
    }

    fn symlink(&self, source: &Path, target: &Path) -> LazinResult<()> {
        let abolute_source =
            fs::canonicalize(source).context("Failed to get absolute path for source")?;

        lazin_logger::info!(
            "Linking {} -> {}",
            abolute_source.display(),
            target.display()
        );

        match replace_with_symlink(&abolute_source, target) {
            Ok(()) => copy_permissions(&abolute_source, target),
            Err(e) if e.kind() == ErrorKind::PermissionDenied => {
                match self.elevator.symlink(&abolute_source, target)? {
                    Performed::Yes => Ok(()),
                    Performed::Skipped => {
                        lazin_logger::warn!(
                            "Skipping linking {} -> {} - target requires elevated permissions",
                            abolute_source.display(),
                            target.display()
                        );

                        Ok(())
                    }
                }
            }
            Err(e) => Err(e).context("Failed to symlink"),
        }
    }
}

pub struct DryRunLinker {
    config: resolve::config::Config,
    filesystem: RefCell<BTreeMap<PathBuf, FileType>>,
    force: bool,
    should_skip_failed_encryption_decryption: bool,
    elevation_policy: ElevationPolicy,
    elevation_check: ElevationCheck,
}

impl DryRunLinker {
    pub fn new(config: resolve::config::Config, linker_options: LinkerOptions) -> Self {
        Self {
            config,
            filesystem: RefCell::default(),
            force: linker_options.force,
            should_skip_failed_encryption_decryption: linker_options
                .should_skip_failed_encryption_decryption,
            elevation_policy: linker_options.elevation_policy,
            elevation_check: ElevationCheck::default(),
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
            should_skip_failed_encryption_decryption: self.should_skip_failed_encryption_decryption,
        };

        link(self, &modules, encrypt_options)?;

        Ok(())
    }

    fn create_dir_all(&self, mut path: &Path) -> LazinResult<Performed> {
        if !path.is_dir() {
            return Ok(Performed::Yes);
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

        Ok(Performed::Yes)
    }

    fn symlink(&self, source: &Path, target: &Path) -> LazinResult<()> {
        match (
            self.elevation_check.requires_elevation(target),
            self.elevation_policy,
        ) {
            (false, _) => {
                lazin_logger::info!("Linking {} -> {}", source.display(), target.display())
            }
            (true, ElevationPolicy::Never) => lazin_logger::warn!(
                "Skipping linking {} -> {} - target requires elevated permissions",
                source.display(),
                target.display()
            ),
            (true, _) => lazin_logger::info!(
                "Linking {} -> {} (requires elevated permissions)",
                source.display(),
                target.display()
            ),
        }

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
    should_skip_failed_encryption_decryption: bool,
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
            if let Performed::Skipped = linker.create_dir_all(target)? {
                lazin_logger::warn!(
                    "Skipping linking {} -> {} - creating the parent directories requires elevated permissions",
                    source.display(),
                    target.display()
                );

                continue;
            }
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
                            if options.should_skip_failed_encryption_decryption
                                && options.encryption_manager.can_decrypt(source)?
                            {
                                lazin_logger::info!(
                                    "Cannot decrypt file {}, skipping",
                                    source.display()
                                );
                                return Ok(());
                            }

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

fn missing_parent_directory(target: &Path) -> Option<&Path> {
    target.parent().filter(|parent_dir| !parent_dir.exists())
}

#[cfg(unix)]
fn replace_with_symlink(source: &Path, target: &Path) -> std::io::Result<()> {
    use std::os::unix::fs::symlink;

    if target.exists() {
        fs::remove_file(target)?;
    }

    symlink(source, target)
}
