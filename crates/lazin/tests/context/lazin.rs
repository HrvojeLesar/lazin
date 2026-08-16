use lazin_test_utils::context::TestContext;

use crate::{
    cmd::{Lazin, LazinFactory},
    context::temp::TempFilesContext,
};

pub struct LazinContext<T: LazinFactory> {
    pub tempfiles_context: TempFilesContext,
    pub lazin: Lazin<T::Output>,
}

impl<T: LazinFactory> TestContext for LazinContext<T> {
    fn setup() -> Self {
        let tempfiles_context = TempFilesContext::setup();
        let dir = Some(tempfiles_context.tempdir.path());
        let lazin = T::new(dir);

        Self {
            tempfiles_context,
            lazin,
        }
    }
}
