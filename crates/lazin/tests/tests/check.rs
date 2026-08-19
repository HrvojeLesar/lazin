use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use lazin_error::LazinResult;
use lazin_test_macros::{lazin_assert, lazin_test};
use lazin_test_utils::expect_ext::ExpectWithContext;

use crate::{
    cmd::LazinFactory,
    context::lazin::{LazinCheckContext, LazinContext},
    tests::{DEFAULT_MODULES_FILE, DEFAULT_WORKSPACE_FILE, setup_empty_lazin_dir},
};

struct ModuleWriter<'a, T>
where
    T: LazinFactory,
{
    module_name: String,
    // TODO: Change pairs to configurable entries closer resembling
    // config data structures
    files: BTreeMap<PathBuf, PathBuf>,
    ctx: &'a LazinContext<T>,
}

impl<'a, T> ModuleWriter<'a, T>
where
    T: LazinFactory,
{
    pub fn new(module_name: impl ToString, ctx: &'a LazinContext<T>) -> Self {
        Self {
            module_name: module_name.to_string(),
            files: BTreeMap::default(),
            ctx,
        }
    }

    pub fn get_writer(&self) -> impl Fn(&mut BufWriter<&File>) -> LazinResult {
        let file_pairs =
            self.files
                .iter()
                .fold(String::new(), |mut output, (source, destination)| {
                    use std::fmt::Write;

                    writeln!(
                        output,
                        "\"{}\" = \"{}\"",
                        source.display(),
                        destination.display()
                    )
                    .expect_with_context("expected a write to string");

                    output
                });

        move |writer| {
            writeln!(
                writer,
                r#"
[{}]
{}
"#,
                self.module_name, file_pairs
            )?;
            Ok(())
        }
    }

    pub fn add(&mut self, source: impl AsRef<Path>, destination: impl AsRef<Path>) {
        let full_source_path = self.ctx.create_path(source);
        let full_destination_path = self.ctx.create_path(destination);

        self.ctx
            .create_file(&full_source_path)
            .expect_with_context(format_args!(
                "failed to create file: '{}'",
                full_source_path.display()
            ));

        self.files.insert(full_source_path, full_destination_path);
    }
}

struct WorkspaceWriter<'a, T>
where
    T: LazinFactory,
{
    workspace_name: String,
    // TODO: Change pairs to configurable entries closer resembling
    // config data structures
    modules: Vec<String>,
    ctx: &'a LazinContext<T>,
}

impl<'a, T> WorkspaceWriter<'a, T>
where
    T: LazinFactory,
{
    pub fn new(wokspace_name: impl ToString, ctx: &'a LazinContext<T>) -> Self {
        Self {
            workspace_name: wokspace_name.to_string(),
            modules: Vec::new(),
            ctx,
        }
    }

    pub fn get_writer(&self) -> impl Fn(&mut BufWriter<&File>) -> LazinResult {
        let modules =
            self.modules
                .iter()
                .enumerate()
                .fold(String::new(), |mut output, (idx, module)| {
                    use std::fmt::Write;

                    if idx == 0 {
                        write!(output, "{module}")
                            .expect_with_context("expected a write to string");
                    } else {
                        write!(output, ",\n{module}")
                            .expect_with_context("expected a write to string");
                    }

                    output
                });

        move |writer| {
            writeln!(
                writer,
                r#"
{} = [
"{}"
]
"#,
                self.workspace_name, modules
            )?;
            Ok(())
        }
    }

    pub fn add(&mut self, module: impl ToString) {
        self.modules.push(module.to_string());
    }
}

fn setup_check_dir<T: LazinFactory>(ctx: &LazinContext<T>) {
    const MODULE_NAME: &str = "test_module";
    const WORKSPACE_NAME: &str = "test_workspace";

    let lazin_dir = setup_empty_lazin_dir(ctx);

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

#[lazin_test(setup_check_dir(ctx))]
fn check(mut ctx: LazinCheckContext) {
    let output = ctx.run();
    let stdout = ctx.stdout(&output);

    lazin_assert!(
        output.status.success(),
        "expected a successfull check status, got: {}, stdout: {}, stderr: {}",
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
