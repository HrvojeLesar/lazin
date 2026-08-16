use std::path::{Path, PathBuf};

use lazin_test_utils::context::TestContext;

use crate::{
    cmd::{Lazin, LazinFactory, init::Init},
    context::temp::TempFilesContext,
};

pub type LazinInitContext = LazinContext<Lazin<Init>>;

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

impl<T: LazinFactory> LazinContext<T> {
    pub fn path(&self) -> &Path {
        self.tempfiles_context.tempdir.path()
    }

    pub fn join_path<P: AsRef<Path>>(&self, other: P) -> PathBuf {
        self.path().join(other)
    }

    pub fn join_path_iter(&self, segments: impl IntoIterator<Item = impl AsRef<Path>>) -> PathBuf {
        let mut path = self.path().to_owned();
        for segment in segments {
            path.push(segment.as_ref());
        }

        path
    }
}
