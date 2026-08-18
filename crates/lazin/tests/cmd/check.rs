use crate::{
    cmd::{Lazin, LazinFactory},
    directory_impl, gitignore_impl,
};

pub struct Check;

impl LazinFactory for Lazin<Check> {
    type Output = Check;
    fn new(dir: Option<&std::path::Path>) -> Lazin<Self::Output> {
        let mut command = crate::cmd::lazin_cmd(dir);
        command.arg("check");
        Self::new_for_t(command)
    }
}

impl Lazin<Check> {
    directory_impl!();
    gitignore_impl!();
}
