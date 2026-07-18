// Source is an encrypted file
// How to get files to encrypt in the first place ???
//      Define a file source=target
//      source is the original file, this creates a source.gpg file
//      add source to .gitignore
// Config resolver should try to decrypt source files

use std::path::Path;

use lazin_error::{Context, LazinResult};
use lazin_gpg_wrapper::{DecryptOptions, EncryptOptions};

use crate::{
    cache::{Cache, Entry, FileHash},
    encryption_management::gitignore::Gitignore,
};

pub mod gitignore;

pub const GPG_EXTENSION: &str = "gpg";

// TODO: move this into resolving step
// when resolving and encountering a non existent encrypted file
// try decrypting file.gpg, it this fails then validation should fail.
// Add a flag for skipping trying to decrypt
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

    pub fn manage_decryption(&mut self, source: &Path) -> LazinResult {
        self.gitignore.managed.insert(source.into());
        self.decrypt(source)?;

        Ok(())
    }

    fn decrypt(&mut self, file: &Path) -> LazinResult<()> {
        let input = file.with_added_extension(GPG_EXTENSION);
        let output = file;

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
}
