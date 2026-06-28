// Source is an encrypted file
// How to get files to encrypt in the first place ???
//      Define a file source=target
//      source is the original file, this creates a source.gpg file
//      add source to .gitignore
// Config resolver should try to decrypt source files

use std::path::Path;

use lazin_error::{Context, LazinResult};
use lazin_gpg_wrapper::DecryptOptions;

use crate::cache::Cache;

const GPG_EXTENSION: &str = "gpg";

pub struct EncryptionManager {
    cache: Cache,
}

impl EncryptionManager {
    pub fn new(cache: Cache) -> Self {
        Self { cache }
    }

    pub fn decrypt(&mut self, file: &Path) -> LazinResult<()> {
        let input = file.with_added_extension(GPG_EXTENSION);
        let output = file;

        lazin_gpg_wrapper::decrypt_file(DecryptOptions {
            input: &input,
            output: file,
        })
        .with_context(|| {
            format!(
                "EncryptionManager failed to decrypt file: {} into {}",
                input.display(),
                output.display()
            )
        })?;

        todo!()
    }
}
