pub struct Unvalidated;
pub struct Passed;
pub struct Failed<E>(pub E);

pub type PipelineResult<E> = Result<(), E>;
pub type ValidatorFn<T, E> = fn(&T) -> PipelineResult<E>;

type Pipeline<T, E> = Result<ValidationStep<T, Passed>, ValidationStep<T, Failed<E>>>;

pub trait Bind<T, E> {
    fn bind(self, validator: ValidatorFn<T, E>) -> Pipeline<T, E>;
    fn result(self) -> PipelineResult<E>;
}

pub struct ValidationStep<T, State = Unvalidated> {
    data: T,
    state: State,
}

impl<T> ValidationStep<T, Unvalidated> {
    pub fn new(data: T) -> Self {
        Self {
            data,
            state: Unvalidated,
        }
    }
}

impl<T, E> Bind<T, E> for Pipeline<T, E> {
    fn bind(self, validator: ValidatorFn<T, E>) -> Pipeline<T, E> {
        let step = self?;
        match validator(&step.data) {
            Ok(_) => Ok(ValidationStep {
                data: step.data,
                state: Passed,
            }),
            Err(e) => Err(ValidationStep {
                data: step.data,
                state: Failed(e),
            }),
        }
    }

    fn result(self) -> PipelineResult<E> {
        match self {
            Ok(_) => Ok(()),
            Err(step) => Err(step.state.0),
        }
    }
}

impl<T, E> Bind<T, E> for ValidationStep<T, Unvalidated> {
    fn bind(self, validator: ValidatorFn<T, E>) -> Pipeline<T, E> {
        match validator(&self.data) {
            Ok(_) => Ok(ValidationStep {
                data: self.data,
                state: Passed,
            }),
            Err(e) => Err(ValidationStep {
                data: self.data,
                state: Failed(e),
            }),
        }
    }

    fn result(self) -> PipelineResult<E> {
        Ok(())
    }
}
