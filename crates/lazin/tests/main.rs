use lazin_test_macros::lazin_test;

use crate::{
    cmd::{Lazin, init::Init},
    context::lazin::LazinContext,
};

mod cmd;
mod context {
    pub mod lazin;
    pub mod temp;
}

#[lazin_test]
fn start(ctx: LazinContext<Lazin<Init>>) {
}
