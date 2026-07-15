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
    dotfiles::resolved_config::ResolvedConfig,
    encryption_management::gitignore::Gitignore,
};

mod gitignore;

const GPG_EXTENSION: &str = "gpg";

pub struct EncryptionManager<'a> {
    cache: Cache,
    resolved_config: &'a ResolvedConfig,
    gitignore: Gitignore,
}

impl<'a> EncryptionManager<'a> {
    pub fn new(cache: Cache, resolved_config: &'a ResolvedConfig, gitignore: Gitignore) -> Self {
        Self {
            cache,
            resolved_config,
            gitignore,
        }
    }

    pub fn do_something(&mut self) -> LazinResult {
        let encrypted_values = self.resolved_config.encrypted_values();
        for value in encrypted_values {
            self.gitignore.managed.insert(value.source.clone());
            let file_hash_changed = self
                .cache
                .add_entry(Entry::Encryption(
                    value.source.clone(),
                    FileHash::hash(&value.source)?,
                ))?
                .is_some();

            if file_hash_changed {
                self.encrypt(value.source)
            }
        }

        Ok(())
    }

    fn decrypt(&mut self, file: &Path) -> LazinResult<()> {
        let input = file.with_added_extension(GPG_EXTENSION);
        let output = file;

        lazin_gpg_wrapper::decrypt_file(DecryptOptions {
            input: &input,
            output: &output,
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
            input: &input,
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
