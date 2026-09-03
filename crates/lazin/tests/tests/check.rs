use lazin_test::expect_ext::ExpectWithContext;
use lazin_test::{lazin_assert, lazin_test};

use crate::{
    cmd::LazinFactory,
    context::lazin::{LazinCheckContext, LazinContext},
    tests::{DEFAULT_MODULES_FILE, DEFAULT_WORKSPACE_FILE, setup_empty_lazin_dir},
    utils::writers::{ModuleWriter, WorkspaceWriter},
};

const UNEXISTING_MODULE: &str = "not_a_module";

fn setup_check_dir<T: LazinFactory>(ctx: &LazinContext<T>, directory_override: Option<&str>) {
    const MODULE_NAME: &str = "test_module";
    const WORKSPACE_NAME: &str = "test_workspace";

    let lazin_dir = setup_empty_lazin_dir(ctx, directory_override);

    let mut module_writer = ModuleWriter::new(MODULE_NAME, ctx);
    module_writer.add("file1", "output_dir/linked_file1");
    let modules_path = lazin_dir.join(DEFAULT_MODULES_FILE);
    ctx.create_file_with_content(modules_path, module_writer.get_writer())
        .expect_with_context("failed to create file with default modules contents");

    let mut workspace_writer = WorkspaceWriter::new(WORKSPACE_NAME, ctx);
    workspace_writer.add(MODULE_NAME);
    let workspace_path = lazin_dir.join(DEFAULT_WORKSPACE_FILE);
    ctx.create_file_with_content(&workspace_path, workspace_writer.get_writer())
        .expect_with_context("failed to create file with default workspace contents");
}

fn setup_invalid_check_dir<T: LazinFactory>(
    ctx: &LazinContext<T>,
    directory_override: Option<&str>,
) {
    const MODULE_NAME: &str = "test_module";
    const WORKSPACE_NAME: &str = "test_workspace";

    let lazin_dir = setup_empty_lazin_dir(ctx, directory_override);

    let mut module_writer = ModuleWriter::new(MODULE_NAME, ctx);
    module_writer.add("file1", "output_dir/linked_file1");
    let modules_path = lazin_dir.join(DEFAULT_MODULES_FILE);
    ctx.create_file_with_content(modules_path, module_writer.get_writer())
        .expect_with_context("failed to create file with default modules contents");

    let mut workspace_writer = WorkspaceWriter::new(WORKSPACE_NAME, ctx);
    workspace_writer.add(MODULE_NAME);
    workspace_writer.add(UNEXISTING_MODULE);
    let workspace_path = lazin_dir.join(DEFAULT_WORKSPACE_FILE);
    ctx.create_file_with_content(&workspace_path, workspace_writer.get_writer())
        .expect_with_context("failed to create file with default workspace contents");
}

#[track_caller]
fn assert_check(ctx: &mut LazinCheckContext) {
    let output = ctx.run();
    let stdout = ctx.stdout(&output);

    lazin_assert!(
        output.status.success(),
        "expected a successfull check status, got: {}, stdout: \n{} stderr: \n{}",
        output.status.success(),
        ctx.stdout(&output),
        ctx.stderr(&output)
    );

    lazin_assert!(
        stdout.contains("valid"),
        "expectd 'valid' in stdout, got: {}",
        stdout
    );
}

#[lazin_test(setup_check_dir(ctx, None))]
fn check(mut ctx: LazinCheckContext) {
    assert_check(&mut ctx);
}

#[lazin_test(setup_check_dir(ctx, Some("check_dir")))]
fn check_custom_dir(mut ctx: LazinCheckContext) {
    ctx.lazin
        .directory("check_dir")
        .expect_with_context("setting a custom directory");
    assert_check(&mut ctx);
}

#[lazin_test(setup_invalid_check_dir(ctx, None))]
fn check_fail(mut ctx: LazinCheckContext) {
    let output = ctx.run();
    let stderr = ctx.stderr(&output);

    let failure = !output.status.success();
    lazin_assert!(
        failure,
        "expected a failed check status, got: {}, stdout: \n{} stderr: \n{}",
        failure,
        ctx.stdout(&output),
        ctx.stderr(&output)
    );

    lazin_assert!(
        stderr.contains(UNEXISTING_MODULE),
        "expectd '{}' in stdout, got: {}",
        UNEXISTING_MODULE,
        stderr
    );
}
