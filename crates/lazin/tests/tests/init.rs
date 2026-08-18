use std::{path::Path, process::Output};

use lazin_test_macros::lazin_test;
use lazin_test_utils::expect_ext::ExpectWithContext;

use crate::{
    cmd::LazinFactory,
    context::lazin::{LazinContext, LazinInitContext},
    tests::{
        DEFAULT_BASE_DIR, DEFAULT_MODULES_FILE, DEFAULT_MODULES_ITER, DEFAULT_WORKSPACE_FILE,
        DEFAULT_WORKSPACE_ITER, setup_empty_lazin_dir,
    },
};

fn assert_output_success(output: &Output) {
    assert!(
        output.status.success(),
        "unexpected exit: '{:#?}'",
        output.status
    );
}

fn assert_path_exists<P: AsRef<Path>>(path: P) {
    let exists = path.as_ref().try_exists().expect_with_context(format_args!(
        "failed to check if path: '{:#?}' exists",
        path.as_ref()
    ));
    assert!(exists, "path does not exist: '{:#?}'", path.as_ref());
}

fn assert_path_does_not_exist<P: AsRef<Path>>(path: P) {
    let exists = path.as_ref().try_exists().expect_with_context(format_args!(
        "failed to check if path: '{:#?}' exists",
        path.as_ref()
    ));
    assert!(!exists, "path exist: '{:#?}'", path.as_ref());
}

fn assert_creates_default_files<T: LazinFactory>(ctx: &mut LazinContext<T>, output: &Output) {
    assert_path_exists(ctx.join_path(DEFAULT_BASE_DIR));
    assert_path_exists(ctx.join_path_iter(DEFAULT_MODULES_ITER));
    assert_path_exists(ctx.join_path_iter(DEFAULT_WORKSPACE_ITER));
    assert_output_success(output);
}

#[lazin_test]
fn init(mut ctx: LazinInitContext) {
    let output = ctx.run();
    assert_creates_default_files(&mut ctx, &output);
}

#[lazin_test(setup_empty_lazin_dir(ctx))]
fn init_on_existing_empty_dir(mut ctx: LazinInitContext) {
    assert_path_exists(ctx.join_path(DEFAULT_BASE_DIR));

    let output = ctx.run();
    assert_creates_default_files(&mut ctx, &output);
}

#[lazin_test]
fn init_on_custom_directory(mut ctx: LazinInitContext) {
    const TEST_DIR: &str = "test_dir";

    assert_path_does_not_exist(TEST_DIR);

    ctx.lazin
        .directory(TEST_DIR)
        .expect("failed to set init directory");

    let output = ctx.run();
    assert_path_exists(ctx.join_path(TEST_DIR));
    assert_path_exists(ctx.join_path_iter([TEST_DIR, DEFAULT_MODULES_FILE]));
    assert_path_exists(ctx.join_path_iter([TEST_DIR, DEFAULT_WORKSPACE_FILE]));
    assert_output_success(&output);
}
