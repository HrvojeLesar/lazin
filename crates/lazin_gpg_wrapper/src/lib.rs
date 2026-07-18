use std::{
    fmt::Display,
    path::Path,
    process::{Command, Stdio},
};

#[derive(Debug)]
pub enum Error {
    GpgNotFound,
    Io(std::io::Error),
    EncryptionFailed(String),
    DecryptionFailed(String),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Io(error) => write!(f, "Io error: {}", error),
            Error::GpgNotFound => write!(f, "Couldn't determine 'gpg' is executable"),
            Error::EncryptionFailed(s) => write!(f, "Encryption failed with error: {}", s),
            Error::DecryptionFailed(s) => write!(f, "Decryption failed with error: {}", s),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::Io(error) => error.source(),
            Error::GpgNotFound => None,
            Error::EncryptionFailed(_) => None,
            Error::DecryptionFailed(_) => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}

pub struct EncryptOptions<'a> {
    pub recipient: &'a str,
    pub input: &'a Path,
    pub output: &'a Path,
}

pub struct DecryptOptions<'a> {
    pub input: &'a Path,
    pub output: &'a Path,
}

pub fn is_gpg_available() -> Result<(), Error> {
    Command::new("gpg")
        .arg("--version")
        .stdout(Stdio::null())
        .status()
        .map_err(|e| match e.kind() {
            std::io::ErrorKind::NotFound => Error::GpgNotFound,
            _ => e.into(),
        })
        .map(|_| ())
}

// TODO: handle keys with passphrase
pub fn encrypt_file(options: EncryptOptions) -> Result<(), Error> {
    is_gpg_available()?;

    let result = Command::new("gpg")
        .arg("--encrypt")
        .arg("--recipient")
        .arg(options.recipient)
        .arg("--output")
        .arg(options.output)
        .arg("--batch")
        .arg("--yes")
        .arg(options.input)
        .output()?;

    match result.status.success() {
        true => Ok(()),
        false => {
            let stderr_string = String::from_utf8_lossy(&result.stderr);
            Err(Error::EncryptionFailed(stderr_string.to_string()))
        }
    }
}

// TODO: handle keys with passphrase
pub fn decrypt_file(options: DecryptOptions) -> Result<(), Error> {
    is_gpg_available()?;

    let result = Command::new("gpg")
        .args(["--decrypt", "--batch", "--yes", "--quiet"])
        .arg("--output")
        .arg(options.output)
        .arg(options.input)
        .output()?;

    match result.status.success() {
        true => Ok(()),
        false => {
            let stderr_string = String::from_utf8_lossy(&result.stderr);
            Err(Error::EncryptionFailed(stderr_string.to_string()))
        }
    }
}
