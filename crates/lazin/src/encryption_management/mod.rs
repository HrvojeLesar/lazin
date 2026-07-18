use std::path::{Path, PathBuf};

use lazin_error::{Context, LazinResult};
use lazin_gpg_wrapper::{DecryptOptions, EncryptOptions};

use crate::{
    cache::{Cache, Entry, FileHash},
    encryption_management::gitignore::Gitignore,
};

pub mod gitignore;

pub const GPG_EXTENSION: &str = "gpg";

// TODO: move this into linking step
// TODO: Add a flag for skipping trying to decrypt
#[derive(Debug)]
pub struct EncryptionManager {
    cache: Cache,
    gitignore: Gitignore,
}

impl EncryptionManager {
    pub fn new(cache: Cache, gitignore: Gitignore) -> Self {
        Self { cache, gitignore }
    }

    pub fn manage_encryption(&mut self, source: &Path, recipient: &str) -> LazinResult {
        self.gitignore.managed.insert(source.into());
        let file_hash_changed = self
            .cache
            .add_entry(Entry::Encryption(source.into(), FileHash::hash(source)?))?
            .is_some();

        if file_hash_changed {
            self.encrypt(source, recipient)?;
        }

        Ok(())
    }

    pub fn manage_decryption(&self, source: &Path, override_output: Option<&Path>) -> LazinResult {
        // TODO: rethink managing. Current system does not work, because
        // it only gitignores files from one workspace not all
        // configured files
        // self.gitignore.managed.insert(source.into());
        self.decrypt(source, override_output)?;

        Ok(())
    }

    fn decrypt(&self, file: &Path, override_output: Option<&Path>) -> LazinResult<()> {
        let input = Self::get_input_file_with_extension(file);
        let output = override_output.unwrap_or(file);

        lazin_gpg_wrapper::decrypt_file(DecryptOptions {
            input: &input,
            output,
        })
        .with_context(|| {
            format!(
                "EncryptionManager failed to decrypt file: {} into {}",
                input.display(),
                output.display()
            )
        })?;

        Ok(())
    }

    fn encrypt(&mut self, file: &Path, recipient: &str) -> LazinResult<()> {
        let input = file;
        let output = file.with_added_extension(GPG_EXTENSION);

        lazin_gpg_wrapper::encrypt_file(EncryptOptions {
            input,
            output: &output,
            recipient,
        })
        .with_context(|| {
            format!(
                "EncryptionManager failed to encrypt file: {} into {}",
                input.display(),
                output.display()
            )
        })?;

        Ok(())
    }

    pub fn get_input_file_with_extension(file: &Path) -> PathBuf {
        file.with_added_extension(GPG_EXTENSION)
    }
}
