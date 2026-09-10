use std::{
    cell::{Cell, RefCell},
    collections::BTreeMap,
    ffi::OsStr,
    io::{self, IsTerminal, Write},
    path::{Path, PathBuf},
    process::Command,
};

use lazin_error::{Context, LazinResult};

use crate::{
    error::LazinError,
    filesystem::link::{ElevationPolicy, Performed},
};

const SUDO: &str = "sudo";
const TEST: &str = "test";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Consent {
    Unasked,
    Granted,
    Denied,
}

#[derive(Debug)]
pub struct Elevator {
    policy: ElevationPolicy,
    consent: Cell<Consent>,
}

impl Elevator {
    pub fn new(policy: ElevationPolicy) -> Self {
        Self {
            policy,
            consent: Cell::new(Consent::Unasked),
        }
    }

    pub fn create_dir_all(&self, directory: &Path) -> LazinResult<Performed> {
        if !self.is_allowed(directory)? {
            return Ok(Performed::Skipped);
        }

        run(&[
            OsStr::new("mkdir"),
            OsStr::new("-p"),
            OsStr::new("--"),
            directory.as_os_str(),
        ])?;

        Ok(Performed::Yes)
    }

    pub fn symlink(&self, source: &Path, target: &Path) -> LazinResult<Performed> {
        if !self.is_allowed(target)? {
            return Ok(Performed::Skipped);
        }

        run(&[
            OsStr::new("ln"),
            OsStr::new("-sfn"),
            OsStr::new("--"),
            source.as_os_str(),
            target.as_os_str(),
        ])?;

        Ok(Performed::Yes)
    }

    fn is_allowed(&self, path: &Path) -> LazinResult<bool> {
        let consent = match (self.policy, self.consent.get()) {
            (ElevationPolicy::Never, _) => Consent::Denied,
            (ElevationPolicy::Always, _) => Consent::Granted,
            (ElevationPolicy::Prompt, Consent::Unasked) => self.ask(path)?,
            (ElevationPolicy::Prompt, answered) => answered,
        };

        self.consent.set(consent);

        Ok(consent == Consent::Granted)
    }

    fn ask(&self, path: &Path) -> LazinResult<Consent> {
        if !io::stdin().is_terminal() {
            lazin_logger::warn!(
                "Elevated permissions are required for '{}', but confirmation cannot be asked for without a terminal; rerun with '--sudo' to link such paths as root",
                path.display()
            );

            return Ok(Consent::Denied);
        }

        lazin_logger::warn!("Elevated permissions are required for '{}'", path.display());
        print!("Link paths requiring elevated permissions as root using '{SUDO}'? [y/N]: ");
        io::stdout()
            .flush()
            .context("Failed to write the elevated permissions prompt")?;

        let mut answer = String::new();
        io::stdin()
            .read_line(&mut answer)
            .context("Failed to read the elevated permissions answer")?;

        Ok(match answer.trim().to_lowercase().as_str() {
            "y" | "yes" => Consent::Granted,
            _ => Consent::Denied,
        })
    }
}

#[derive(Debug, Default)]
pub struct ElevationCheck {
    directories: RefCell<BTreeMap<PathBuf, bool>>,
}

impl ElevationCheck {
    pub fn requires_elevation(&self, target: &Path) -> bool {
        let Some(directory) = target.parent().and_then(nearest_existing_directory) else {
            return false;
        };

        if let Some(requires_elevation) = self.directories.borrow().get(directory).copied() {
            return requires_elevation;
        }

        let requires_elevation = matches!(is_writable(directory), Some(false));
        self.directories
            .borrow_mut()
            .insert(directory.into(), requires_elevation);

        requires_elevation
    }
}

fn nearest_existing_directory(path: &Path) -> Option<&Path> {
    path.ancestors()
        .find(|ancestor| ancestor.is_dir())
        .or_else(|| path.is_relative().then_some(Path::new(".")))
}

fn is_writable(directory: &Path) -> Option<bool> {
    let directory = directory.as_os_str();
    let status = Command::new(TEST)
        .args([
            OsStr::new("-w"),
            directory,
            OsStr::new("-a"),
            OsStr::new("-x"),
            directory,
        ])
        .status()
        .ok()?;

    Some(status.success())
}

fn run(arguments: &[&OsStr]) -> LazinResult<()> {
    let command_line = command_line(arguments);
    lazin_logger::info!("Running '{}'", command_line);

    let status = Command::new(SUDO)
        .arg("--")
        .args(arguments)
        .status()
        .map_err(|e| match e.kind() {
            io::ErrorKind::NotFound => LazinError::SudoNotFound,
            _ => LazinError::Io(e),
        })?;

    match status.success() {
        true => Ok(()),
        false => Err(LazinError::PrivilegedCommandFailed(format!(
            "'{}' exited with {}",
            command_line, status
        )))
        .context("Failed to run a privileged command"),
    }
}

fn command_line(arguments: &[&OsStr]) -> String {
    let mut command_line = format!("{} --", SUDO);
    for argument in arguments {
        command_line.push(' ');
        command_line.push_str(&argument.to_string_lossy());
    }

    command_line
}

#[cfg(test)]
mod test {
    use std::path::Path;

    use crate::filesystem::privileged::{ElevationCheck, nearest_existing_directory};

    #[test]
    fn nearest_existing_directory_of_a_missing_path() {
        assert_eq!(
            nearest_existing_directory(Path::new("/etc/lazin/missing/directory")),
            Some(Path::new("/etc"))
        );
    }

    #[test]
    fn nearest_existing_directory_of_a_relative_path() {
        assert_eq!(
            nearest_existing_directory(Path::new("missing/directory")),
            Some(Path::new("."))
        );
    }

    #[test]
    fn home_directory_does_not_require_elevation() {
        let Some(home_directory) = std::env::var_os("HOME") else {
            return;
        };

        assert!(
            !ElevationCheck::default()
                .requires_elevation(&Path::new(&home_directory).join("lazin_link"))
        );
    }

    #[test]
    fn a_directory_is_only_checked_once() {
        let check = ElevationCheck::default();
        let target = Path::new("/etc/lazin_link");

        let requires_elevation = check.requires_elevation(target);

        assert_eq!(check.requires_elevation(target), requires_elevation);
        assert_eq!(check.directories.borrow().len(), 1);
    }
}
