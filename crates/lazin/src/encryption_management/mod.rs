use std::{
    cell::RefCell,
    path::{Path, PathBuf},
};

use lazin_error::{Context, LazinResult};
use lazin_gpg_wrapper::{DecryptOptions, EncryptOptions};

use crate::cache::{Cache, Entry, FileHash};

pub mod gitignore;

pub const GPG_EXTENSION: &str = "gpg";

// TODO: Add a flag for skipping trying to decrypt or continue linking if
// decryption fails
#[derive(Debug)]
pub struct EncryptionManager {
    cache: RefCell<Cache>,
}

impl EncryptionManager {
    pub fn new(cache: Cache) -> Self {
        Self {
            cache: RefCell::new(cache),
        }
    }

    pub fn manage_encryption(&self, source: &Path, recipient: &str) -> LazinResult<bool> {
        let file_hash = FileHash::hash(source)?;
        let previous_entry = self
            .cache
            .borrow_mut()
            .add_entry(Entry::Encryption(source.into(), file_hash.clone()))?;

        let file_hash_changed = match previous_entry {
            Some(e) => match e {
                Entry::Encryption(_, old_hash) => file_hash != old_hash,
            },
            None => true,
        };

        let did_encrypt_file = if file_hash_changed || !Self::encrypted_file_exists(source) {
            self.encrypt(source, recipient)?;
            true
        } else {
            false
        };

        Ok(did_encrypt_file)
    }

    pub fn manage_decryption(&self, source: &Path, override_output: Option<&Path>) -> LazinResult {
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

    fn encrypt(&self, file: &Path, recipient: &str) -> LazinResult<()> {
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

    pub fn flush_cache(&self) -> LazinResult {
        self.cache.borrow().save()
    }

    fn encrypted_file_exists(file: &Path) -> bool {
        Self::get_input_file_with_extension(file).exists()
    }
}
