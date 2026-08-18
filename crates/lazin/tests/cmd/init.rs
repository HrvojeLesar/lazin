use crate::{
    cmd::{Lazin, LazinFactory},
    directory_impl, impl_lazin_factory,
};

pub struct Init;
impl_lazin_factory!(Init, "init");

impl Lazin<Init> {
    directory_impl!();
}
