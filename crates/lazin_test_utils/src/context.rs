use std::ops::{Deref, DerefMut};

pub trait TestContext
where
    Self: Sized,
{
    fn setup() -> Self;
    fn teardown(&mut self) {}
}

pub struct TestContextDropGuard<T: TestContext>(pub T);

impl<T: TestContext> Drop for TestContextDropGuard<T> {
    fn drop(&mut self) {
        self.0.teardown();
    }
}

impl<T: TestContext> Deref for TestContextDropGuard<T> {
    type Target = T;
    fn deref(&self) -> &T {
        &self.0
    }
}

impl<T: TestContext> DerefMut for TestContextDropGuard<T> {
    fn deref_mut(&mut self) -> &mut T {
        &mut self.0
    }
}
