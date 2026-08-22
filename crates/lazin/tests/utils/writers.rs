use std::{
    collections::BTreeMap,
    fs::File,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use lazin_error::LazinResult;
use lazin_test_utils::expect_ext::ExpectWithContext;

use crate::{cmd::LazinFactory, context::lazin::LazinContext};

pub struct WorkspaceWriter<'a, T>
where
    T: LazinFactory,
{
    workspace_name: String,
    // TODO: Change pairs to configurable entries closer resembling
    // config data structures
    modules: Vec<String>,
    #[allow(unused)]
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
                        write!(output, "\"{module}\"")
                            .expect_with_context("expected a write to string");
                    } else {
                        write!(output, ",\n\"{module}\"")
                            .expect_with_context("expected a write to string");
                    }

                    output
                });

        move |writer| {
            writeln!(
                writer,
                r#"
{} = [
{}
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

pub struct ModuleWriter<'a, T>
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
