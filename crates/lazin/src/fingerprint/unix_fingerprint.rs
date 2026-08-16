use lazin_error::{Context, LazinError, LazinResult};

use crate::fingerprint::FingerprintParts;

const MACHINE_ID_FILE: &str = "/etc/machine-id";

#[derive(Debug, Clone)]
pub struct UnixFingerprint;

const fn failed_to_read_machine_id_file() -> &'static str {
    // NOTE: MACHINE_ID_FILE cannot be used directly, `concat!()` can only use literals
    // and `stringify!()` converts the variable name into a string
    concat!("Failed to read '", "/etc/machine-id", "' file")
}

impl UnixFingerprint {
    pub fn new() -> Self {
        UnixFingerprint
    }

    fn get_machine_id() -> LazinResult<String> {
        std::fs::read_to_string(MACHINE_ID_FILE).context(failed_to_read_machine_id_file())
    }
}

impl FingerprintParts for UnixFingerprint {
    type Error = LazinError;

    fn get_user_id(&self) -> Result<String, Self::Error> {
        let user_id = unsafe { libc::getuid() }.to_string();

        Ok(user_id)
    }

    fn get_machine_id(&self) -> Result<String, Self::Error> {
        Self::get_machine_id()
    }
}

#[cfg(test)]
mod test {
    use crate::fingerprint::{Fingerprint, unix_fingerprint::UnixFingerprint};

    #[test]
    fn can_get_fingerprint() {
        let unix_fingerprint = UnixFingerprint::new();
        unix_fingerprint
            .fingerprint()
            .expect("failed to get fingerprint");
    }
}
