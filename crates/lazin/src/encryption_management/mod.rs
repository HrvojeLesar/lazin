// Source is an encrypted file
// How to get files to encrypt in the first place ???
//      Define a file source=target
//      source is the original file, this creates a source.gpg file
//      add source to .gitignore
// Config resolver should try to decrypt source files

use std::path::{Path, PathBuf};

use lazin_gpg_wrapper::DecryptOptions;

use crate::error::{Context, LazinResult};

const GPG_EXTENSION: &str = "gpg";

pub struct EncryptionManager {
    cache_dir: PathBuf,
}

impl EncryptionManager {
    pub fn new<T: Into<PathBuf>>(cache_dir: T) -> Self {
        Self {
            cache_dir: cache_dir.into(),
        }
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
