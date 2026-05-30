use std::{
    env,
    fs::{self, OpenOptions},
    path::{Path, PathBuf},
    sync::atomic::AtomicUsize,
};

static FILE_COUNT: AtomicUsize = AtomicUsize::new(0);

fn next_file() -> String {
    format!(
        "{}",
        FILE_COUNT.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    )
}

pub struct TempFilepath(PathBuf);

impl TempFilepath {
    pub fn new() -> Self {
        let file_path = env::temp_dir().join(next_file());
        OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&file_path)
            .expect("a valid new file");

        Self(file_path)
    }

    pub fn new_with_parent_dir(parent_dir: &Path) -> Self {
        assert!(parent_dir.is_dir(), "not a directory");

        Self(parent_dir.join(next_file()))
    }

    pub fn path(&self) -> &Path {
        self.0.as_path()
    }
}

impl Drop for TempFilepath {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.0);
    }
}

pub struct TempDir(PathBuf);

impl TempDir {
    pub fn new() -> Self {
        let dir_path = env::temp_dir().join(next_file());
        fs::create_dir(&dir_path).expect("a valid temp directory path");
        Self(dir_path)
    }

    pub fn path(&self) -> &Path {
        self.0.as_path()
    }
}

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}
