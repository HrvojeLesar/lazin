use std::path::PathBuf;

use lazin_test::expect_ext::ExpectWithContext;

use crate::{cmd::LazinFactory, context::lazin::LazinContext};

mod check;
mod init;

pub const DEFAULT_BASE_DIR: &str = "lazin";
pub const DEFAULT_MODULES_FILE: &str = "modules.toml";
pub const DEFAULT_WORKSPACE_FILE: &str = "workspace.toml";
pub const DEFAULT_MODULES_ITER: [&str; 2] = [DEFAULT_BASE_DIR, DEFAULT_MODULES_FILE];
pub const DEFAULT_WORKSPACE_ITER: [&str; 2] = [DEFAULT_BASE_DIR, DEFAULT_WORKSPACE_FILE];

pub fn setup_empty_lazin_dir<T: LazinFactory>(
    ctx: &LazinContext<T>,
    directory_override: Option<&str>,
) -> PathBuf {
    ctx.create_dir(directory_override.unwrap_or(DEFAULT_BASE_DIR))
        .expect_with_context(format_args!(
            "expected an empty '{DEFAULT_BASE_DIR}' directory"
        ))
}
