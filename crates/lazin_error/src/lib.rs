use std::fmt::{Debug, Display};

pub type LazinResult<T = (), E = LazinError> = Result<T, E>;

use std::any::{Any, TypeId};
use std::error::Error as StdError;

#[derive(Debug)]
pub struct LazinError(Box<dyn Walk>);

impl LazinError {
    fn new<E>(error: E) -> Self
    where
        E: StdError + Send + Sync + 'static,
    {
        LazinError(Box::new(WalkableErrorWrapper(error)))
    }

    fn from_context<C, E>(context: C, error: E) -> Self
    where
        C: Debug + Display + Send + Sync + 'static,
        E: Debug + Display + Send + Sync + 'static,
    {
        LazinError(Box::new(LazinContextError { context, error }))
    }

    pub fn is<E: 'static>(&self) -> bool {
        self.downcast_ref::<E>().is_some()
    }

    pub fn downcast_ref<E: 'static>(&self) -> Option<&E> {
        self.0.walk(TypeId::of::<E>())?.downcast_ref()
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

trait Walk: StdError + Send + Sync + 'static {
    fn walk(&self, target: TypeId) -> Option<&dyn Any>;
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

impl<C, E> Walk for LazinContextError<C, E>
where
    C: Debug + Display + Send + Sync + 'static,
    E: Debug + Display + Send + Sync + 'static,
{
    fn walk(&self, target: TypeId) -> Option<&dyn Any> {
        if TypeId::of::<C>() == target {
            return Some(&self.context as &dyn Any);
        }

        if TypeId::of::<E>() == target {
            return Some(&self.error as &dyn Any);
        }

        (&self.error as &dyn Any)
            .downcast_ref::<LazinError>()?
            .0
            .walk(target)
    }
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
            Err(error) => Err(LazinError::from_context(context, error)),
        }
    }

    fn with_context<C, F: FnOnce() -> C>(self, context: F) -> Result<T, LazinError>
    where
        C: Display + Debug + Send + Sync + 'static,
    {
        match self {
            Ok(o) => Ok(o),
            Err(error) => Err(LazinError::from_context(context(), error)),
        }
    }
}

struct WalkableErrorWrapper<E>(E);

impl<E> Debug for WalkableErrorWrapper<E>
where
    E: Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Debug::fmt(&self.0, f)
    }
}

impl<E> Display for WalkableErrorWrapper<E>
where
    E: Display,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        Display::fmt(&self.0, f)
    }
}

impl<E> StdError for WalkableErrorWrapper<E>
where
    E: StdError + Send + Sync + 'static,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        self.0.source()
    }
}

impl<E> Walk for WalkableErrorWrapper<E>
where
    E: StdError + Send + Sync + 'static,
{
    fn walk(&self, target: TypeId) -> Option<&dyn Any> {
        (TypeId::of::<E>() == target).then_some(&self.0 as &dyn Any)
    }
}

impl From<LazinError> for Box<dyn StdError + Send + Sync + 'static> {
    fn from(value: LazinError) -> Self {
        value.0
    }
}
