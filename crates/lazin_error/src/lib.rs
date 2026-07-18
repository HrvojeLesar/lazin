use std::fmt::{Debug, Display};

pub type LazinResult<T = (), E = LazinError> = Result<T, E>;

use std::error::Error as StdError;

#[derive(Debug)]
pub struct LazinError(Box<dyn StdError + Send + Sync + 'static>);

impl LazinError {
    fn new<E>(error: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        LazinError(Box::new(error))
    }
}

impl<E> From<E> for LazinError
where
    E: StdError + Send + Sync + 'static,
{
    fn from(value: E) -> Self {
        LazinError::new(value)
    }
}

impl Display for LazinError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

#[derive(Debug)]
struct LazinContextError<C, E> {
    context: C,
    error: E,
}

impl<C, E> Display for LazinContextError<C, E>
where
    C: Display,
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.context, self.error)
    }
}

impl<C, E> StdError for LazinContextError<C, E>
where
    C: Debug + Display,
    E: Debug + Display,
{
}

pub trait Context<T> {
    fn context<C>(self, context: C) -> Result<T, LazinError>
    where
        C: Display + Debug + Send + Sync + 'static;
    fn with_context<C, F: FnOnce() -> C>(self, context: F) -> Result<T, LazinError>
    where
        C: Display + Debug + Send + Sync + 'static;
}

impl<T, E> Context<T> for Result<T, E>
where
    E: Display + Debug + Send + Sync + 'static,
{
    fn context<C>(self, context: C) -> Result<T, LazinError>
    where
        C: Display + Debug + Send + Sync + 'static,
    {
        match self {
            Ok(o) => Ok(o),
            Err(error) => Err(LazinError::new(LazinContextError { context, error })),
        }
    }

    fn with_context<C, F: FnOnce() -> C>(self, context: F) -> Result<T, LazinError>
    where
        C: Display + Debug + Send + Sync + 'static,
    {
        match self {
            Ok(o) => Ok(o),
            Err(error) => Err(LazinError::new(LazinContextError {
                context: context(),
                error,
            })),
        }
    }
}
