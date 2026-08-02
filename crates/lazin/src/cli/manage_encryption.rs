use std::path::PathBuf;

use clap::Args;
use lazin_error::LazinResult;

use crate::{
    common::{self},
    resolve,
};

/// By default reencrypts files and adds them to `lazin.cache`.
/// Can also be used to decrypt configured module files
#[derive(Args)]
pub(super) struct ManageEncryption {
    #[arg(short = 'd', long = "directory", help = "`lazin` config directory")]
    directory: Option<PathBuf>,
    #[arg(
        short = 'r',
        long = "decrypt",
        help = "Decrypt files from all configured modules"
    )]
    decrypt: bool,
    #[arg(
        short = 's',
        long = "skip-failed",
        help = "Skips any files that fail encryption/decryption, by default failure will stop the process partially"
    )]
    skip_failed: bool,
}

impl ManageEncryption {
    pub(super) fn manage(&self) -> LazinResult<()> {
        if !self.decrypt {
            self.reencrypt()
        } else {
            self.decrypt()
        }
    }

    fn reencrypt(&self) -> LazinResult {
        let config = common::parse_config(self.directory.as_deref())?;

        config
            .expanded_modules
            .iter()
            .try_for_each(|m| -> LazinResult {
                m.values.iter().try_for_each(|v| -> LazinResult {
                    if let resolve::module::Encryption::Enabled { recipient } = &v.encryption {
                        if self.skip_failed
                            && config
                                .encryption_manager
                                .can_encrypt(&v.source, recipient)?
                        {
                            lazin_logger::info!(
                                "Cannot encrypt file {}, skipping",
                                v.source.display()
                            );
                            return Ok(());
                        }
                        let did_encrypt = config
                            .encryption_manager
                            .manage_encryption(&v.source, recipient)?;
                        if did_encrypt {
                            lazin_logger::info!("Encrypted file {}", v.source.display())
                        }
                    };

                    Ok(())
                })
            })?;

        config.encryption_manager.flush_cache()?;

        Ok(())
    }

    fn decrypt(&self) -> LazinResult {
        let config = common::parse_config(self.directory.as_deref())?;

        config
            .expanded_modules
            .iter()
            .try_for_each(|m| -> LazinResult {
                m.values.iter().try_for_each(|v| -> LazinResult {
                    if let resolve::module::Encryption::Enabled { .. } = &v.encryption {
                        if self.skip_failed && config.encryption_manager.can_decrypt(&v.source)? {
                            lazin_logger::info!(
                                "Cannot decrypt file {}, skipping",
                                v.source.display()
                            );
                            return Ok(());
                        }
                        config
                            .encryption_manager
                            .manage_decryption(&v.source, None)?;

                        lazin_logger::info!("Decrypted file {}", v.source.display())
                    };

                    Ok(())
                })
            })?;

        config.encryption_manager.flush_cache()?;

        Ok(())
    }
}
