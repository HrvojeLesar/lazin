use crate::error::Error;

pub struct TomlDiagnostic<'a> {
    filename: &'a str,
    source: &'a str,
    error: &'a Error,
}

impl<'a> TomlDiagnostic<'a> {
    pub fn new(filename: &'a str, source: &'a str, error: &'a Error) -> Self {
        Self {
            filename,
            source,
            error,
        }
    }

    pub fn emit(&self) {}
}
