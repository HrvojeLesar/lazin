use std::marker::PhantomData;

pub struct Unvalidated;
pub struct Passed;
pub struct FailedStep<E>(pub E);

pub type PipelineResult<E> = Result<(), E>;
type Pipeline<T, E> = Result<Step<T, Passed>, FailedStep<E>>;

pub trait Bind<T, E> {
    fn bind<F>(self, validator: F) -> Pipeline<T, E>
    where
        F: Fn(&T) -> PipelineResult<E>;
    fn result(self) -> PipelineResult<E>;
}

pub struct Step<T, State = Unvalidated> {
    data: T,
    state: PhantomData<State>,
}

impl<T> Step<T, Unvalidated> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            state: PhantomData,
        }
    }
}

pub fn new<T>(data: T) -> Step<T, Unvalidated> {
    Step::new(data)
}

impl<T, E> Bind<T, E> for Pipeline<T, E> {
    fn bind<F>(self, validator: F) -> Pipeline<T, E>
    where
        F: Fn(&T) -> PipelineResult<E>,
    {
        let step = self?;
        match validator(&step.data) {
            Ok(_) => Ok(Step {
                data: step.data,
                state: PhantomData,
            }),
            Err(e) => Err(FailedStep(e)),
        }
    }

    fn result(self) -> PipelineResult<E> {
        match self {
            Ok(_) => Ok(()),
            Err(step) => Err(step.0),
        }
    }
}

impl<T, E> Bind<T, E> for Step<T, Unvalidated> {
    fn bind<F>(self, validator: F) -> Pipeline<T, E>
    where
        F: Fn(&T) -> PipelineResult<E>,
    {
        match validator(&self.data) {
            Ok(_) => Ok(Step {
                data: self.data,
                state: PhantomData,
            }),
            Err(e) => Err(FailedStep(e)),
        }
    }

    fn result(self) -> PipelineResult<E> {
        Ok(())
    }
}
