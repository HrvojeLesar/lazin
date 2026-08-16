use lazin_test_utils::{context::TestContext, temp::TempDir};

pub struct TempFilesContext {
    pub tempdir: TempDir,
}

impl TestContext for TempFilesContext {
    fn setup() -> Self {
        let tempdir = TempDir::new();

        Self { tempdir }
    }
}
