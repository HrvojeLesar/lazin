use lazin_test::{context::TestContext, temp::TempDir};

pub struct TempFilesContext {
    pub tempdir: TempDir,
}

impl TestContext for TempFilesContext {
    fn setup() -> Self {
        let tempdir = TempDir::new();

        Self { tempdir }
    }
}
