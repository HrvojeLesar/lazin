use std::path::Path;

use lazin_error::LazinResult;

use crate::cmd::{Lazin, LazinFactory, lazin_cmd};

pub struct Init;

impl LazinFactory for Lazin<Init> {
    type Output = Init;

    fn new(dir: Option<&Path>) -> Lazin<Self::Output> {
        let mut command = lazin_cmd(dir);
        command.arg("init");

        Self::new_for_t(command)
    }
}

impl Lazin<Init> {
    pub fn directory<P: AsRef<Path>>(&mut self, directory: P) -> LazinResult<()> {
        self.command.arg("--directory").arg(directory.as_ref());
        Ok(())
    }
}
