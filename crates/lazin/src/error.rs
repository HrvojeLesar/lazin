use std::fmt::Display;

#[derive(Debug)]
pub enum Error {
    TomlParse(toml::de::Error),
    Io(std::io::Error),
    Custom(&'static str),
}

impl Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TomlParse(error) => error.fmt(f),
            Error::Io(error) => error.fmt(f),
            Error::Custom(c) => write!(f, "{}", c),
        }
    }
}

impl From<toml::de::Error> for Error {
    fn from(value: toml::de::Error) -> Self {
        Self::TomlParse(value)
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::Io(value)
    }
}
