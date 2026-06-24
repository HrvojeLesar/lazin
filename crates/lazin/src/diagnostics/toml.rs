use crate::error::LazinError;

pub struct TomlDiagnostic<'a> {
    filename: &'a str,
    source: &'a str,
    error: &'a LazinError,
}

impl<'a> TomlDiagnostic<'a> {
    pub fn new(filename: &'a str, source: &'a str, error: &'a LazinError) -> Self {
        Self {
            filename,
            source,
            error,
        }
    }

    pub fn emit(&self) {}
}
