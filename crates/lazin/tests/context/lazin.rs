use std::{
    fs::File,
    io::{BufWriter, Seek, Write},
    path::{Path, PathBuf},
    process::Output,
};

use lazin_error::{Context, LazinResult};
use lazin_test::context::TestContext;

use crate::{
    cmd::{Lazin, LazinFactory, check::Check, init::Init},
    context::temp::TempFilesContext,
};

pub type LazinInitContext = LazinContext<Lazin<Init>>;
pub type LazinCheckContext = LazinContext<Lazin<Check>>;

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

    pub fn run(&mut self) -> Output {
        self.lazin.output()
    }

    pub fn create_file<P: AsRef<Path>>(&self, path: P) -> LazinResult<std::fs::File> {
        let path = self.create_path(path);
        std::fs::File::create_new::<_>(&path)
            .with_context(|| format!("failed to create new file '{}'", path.display()))
    }

    pub fn create_file_with_content<P, F>(
        &self,
        path: P,
        write_func: F,
    ) -> LazinResult<std::fs::File>
    where
        P: AsRef<Path>,
        F: Fn(&mut BufWriter<&File>) -> LazinResult,
    {
        let path = self.create_path(path);
        let file = self.create_file(&path)?;
        {
            let mut writer = BufWriter::new(&file);
            write_func(&mut writer)
                .with_context(|| format!("failed to write to file '{}'", path.display()))?;

            writer
                .flush()
                .with_context(|| format!("failed to flush file '{}'", path.display()))?;
        }

        let mut file = file;
        file.rewind()?;

        Ok(file)
    }

    pub fn create_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        if path.as_ref().starts_with(self.path()) {
            path.as_ref().into()
        } else {
            self.path().join(path)
        }
    }

    pub fn create_dir<P: AsRef<Path>>(&self, dirname: P) -> LazinResult<PathBuf> {
        let path = self.create_path(dirname);
        std::fs::create_dir(&path)
            .with_context(|| format!("failed to create directory at '{}'", path.display()))?;

        Ok(path)
    }

    pub fn stdout(&self, output: &Output) -> String {
        String::from_utf8_lossy(&output.stdout).to_string()
    }

    pub fn stderr(&self, output: &Output) -> String {
        String::from_utf8_lossy(&output.stderr).to_string()
    }
}
