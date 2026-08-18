use std::{
    marker::PhantomData,
    path::Path,
    process::{Command, Output},
};

pub mod init;

#[inline]
pub fn lazin_cmd(dir: Option<&Path>) -> Command {
    let mut command = Command::new(env!("CARGO_BIN_EXE_lazin"));
    if let Some(dir) = dir {
        command.current_dir(dir);
    }

    command
}

pub trait LazinFactory {
    type Output;

    fn new(dir: Option<&Path>) -> Lazin<Self::Output>;
}

pub struct Lazin<T> {
    state: PhantomData<T>,
    command: Command,
}

impl<T> Lazin<T> {
    fn new_for_t(command: Command) -> Lazin<T> {
        Self {
            state: PhantomData,
            command,
        }
    }

    pub fn output(&mut self) -> Output {
        self.command.output().expect("Failed to execute lazin")
    }
}

#[macro_export]
macro_rules! directory_impl {
    () => {
        pub fn directory<P: AsRef<std::path::Path>>(
            &mut self,
            directory: P,
        ) -> lazin_error::LazinResult<()> {
            self.command.arg("--directory").arg(directory.as_ref());
            Ok(())
        }
    };
}

#[macro_export]
macro_rules! impl_lazin_factory {
    ($ty:ty, $($cmd:literal),+ $(,)?) => {
        impl LazinFactory for Lazin<$ty> {
            type Output = $ty;

            fn new(dir: Option<&std::path::Path>) -> Lazin<Self::Output> {
                let mut command = $crate::cmd::lazin_cmd(dir);
                $(command.arg($cmd);)+

                Self::new_for_t(command)
            }
        }
    };
}

#[macro_export]
macro_rules! gitignore_impl {
    () => {
        pub fn gitignore<P: AsRef<std::path::Path>>(
            &mut self,
            directory: P,
        ) -> lazin_error::LazinResult<()> {
            self.command.arg("--gitignore").arg(directory.as_ref());
            Ok(())
        }
    };
}
