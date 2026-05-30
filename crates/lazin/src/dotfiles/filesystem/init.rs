use std::{
    fs::OpenOptions,
    io::{ErrorKind, Write},
    path::Path,
};

use crate::dotfiles::error::Error;

const LAZIN_DEFAULT_WORKSPACE_FILE: &str = "workspace.toml";
const LAZIN_DEFAULT_MODULE_FILE: &str = "modules.toml";

const LAZIN_DEFAULT_WORKSPACE: &str = r#"
# workspace_name = [
# "module1",
# "module2",
# ]
"#;

const LAZIN_DEFAULT_MODULE: &str = r#"
# [my_module]
# source_file="/.config/my_program/source_file"
# source_dir="/.config/source_dir"
# "more/specific/path/to/source_file"="/.config/my_program/source_file"
# source_file_with_attributes = {
#   path = "/.config/my_program/source_file_with_attributes"
#   encrypt = false # encrypt is optional
# }
"#;

enum WriteFileState {
    Written,
    Skipped,
}

fn try_write_file(base_dir: &Path, filename: &str, data: &[u8]) -> Result<WriteFileState, Error> {
    let mut write_state = WriteFileState::Written;

    let new_file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(base_dir.to_path_buf().join(filename));

    if let Ok(mut file) = new_file {
        if let Err(e) = file.write_all(data) {
            return Err(e.into());
        }
    } else if let Err(e) = new_file {
        match e.kind() {
            ErrorKind::AlreadyExists => write_state = WriteFileState::Skipped,
            _ => return Err(e.into()),
        }
    }

    Ok(write_state)
}

pub fn init_default_config(base_dir: &Path) -> Result<(), Error> {
    let lazin_default_workspace = try_write_file(
        base_dir,
        LAZIN_DEFAULT_WORKSPACE_FILE,
        LAZIN_DEFAULT_WORKSPACE.as_bytes(),
    )?;
    match lazin_default_workspace {
        WriteFileState::Written => (),
        WriteFileState::Skipped => lazin_logger::info!(
            "Skipped writing {} file. File already exists.",
            LAZIN_DEFAULT_WORKSPACE_FILE
        ),
    }

    let lazin_default_module = try_write_file(
        base_dir,
        LAZIN_DEFAULT_MODULE_FILE,
        LAZIN_DEFAULT_MODULE.as_bytes(),
    )?;
    match lazin_default_module {
        WriteFileState::Written => (),
        WriteFileState::Skipped => lazin_logger::info!(
            "Skipped writing {} file. File already exists.",
            LAZIN_DEFAULT_MODULE_FILE
        ),
    }

    Ok(())
}

#[cfg(test)]
mod test {
    use crate::dotfiles::filesystem::{init::init_default_config, tmp::TempDir};

    #[test]
    fn temp_default_config() {
        let temp_dir = TempDir::new();
        init_default_config(temp_dir.path()).expect("initialized config file examples")
    }
}
