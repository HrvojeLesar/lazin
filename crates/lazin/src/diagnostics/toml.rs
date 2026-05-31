use codespan_reporting::{
    diagnostic::{Diagnostic, Label},
    files::SimpleFile,
    term::{
        self,
        termcolor::{ColorChoice, StandardStream},
    },
};

use crate::dotfiles::error::Error;

struct TomlDiagnostic<'a> {
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

    pub fn emit(&self) {
        if let Error::TomlParse(error) = self.error {
            let span = error.span().unwrap();
            let file = SimpleFile::new("raw", self.source);

            let diagnostic = Diagnostic::error()
                .with_message(error.message())
                .with_label(Label::primary((), span));

            let mut writer = StandardStream::stderr(ColorChoice::Auto);
            let config = codespan_reporting::term::Config::default();

            term::emit_to_write_style(&mut writer, &config, &file, &diagnostic).unwrap();
        }
    }
}
