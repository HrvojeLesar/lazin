use lazin_error::{LazinError, LazinResult};

#[cfg(unix)]
pub mod unix_fingerprint;

pub trait FingerprintParts {
    type Error: Into<LazinError>;

    fn get_user_id(&self) -> Result<String, Self::Error>;
    fn get_machine_id(&self) -> Result<String, Self::Error>;
}

pub trait Fingerprint: FingerprintParts {
    fn fingerprint(&self) -> LazinResult<String> {
        let user_id = self.get_user_id().map_err(Into::into)?;
        let machine_id = self.get_machine_id().map_err(Into::into)?;

        Ok(format!("{}-{}", user_id, machine_id))
    }
}

impl<T: FingerprintParts> Fingerprint for T {}
