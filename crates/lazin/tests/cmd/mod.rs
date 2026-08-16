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
