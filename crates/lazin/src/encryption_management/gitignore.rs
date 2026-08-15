use std::{
    collections::BTreeSet,
    fs,
    io::{BufWriter, Write},
    path::{Path, PathBuf},
};

use lazin_error::{Context, LazinResult};

const BEGIN_MARKER: &str = concat!(
    "# >>> lazin v",
    env!("CARGO_PKG_VERSION_MAJOR"),
    " - PLEASE DO NOT MODIFY MANUALLY >>>"
);
const END_MARKER: &str = "# <<< lazin - PLEASE DO NOT MODIFY MANUALLY <<<";

/// Does not load with any managed paths so they can edited on every lazin invocation
#[derive(Debug)]
pub struct Gitignore {
    pub managed: BTreeSet<PathBuf>,
    user_content: String,
    path: PathBuf,
}

impl Gitignore {
    pub fn load(path: impl AsRef<Path>) -> LazinResult<Self> {
        fn split(contents: String) -> String {
            let start = contents.find(BEGIN_MARKER);
            let end = contents.find(END_MARKER);

            match (start, end) {
                (Some(start), Some(end)) if start < end => {
                    let user_content_before_marker = &contents[..start];
                    let user_content_after_marker = &contents[end + END_MARKER.len()..];
                    let user_content = format!(
                        "{}{}",
                        user_content_before_marker.trim_end(),
                        user_content_after_marker
                    );

                    user_content
                }
                _ => contents,
            }
        }

        let path = path.as_ref().to_path_buf();
        let contents = fs::read_to_string(&path).unwrap_or_default();

        let user_content = split(contents);

        Ok(Self {
            managed: BTreeSet::default(),
            user_content,
            path,
        })
    }

    pub fn save(&self) -> LazinResult {
        let file = fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(&self.path)
            .context("Failed to open gitignore file")?;
        let mut writer = BufWriter::new(&file);

        writeln!(writer, "{}", BEGIN_MARKER).context("Failed to write begin marker")?;
        for managed in &self.managed {
            writeln!(writer, "{}", managed.display()).context("Failed to write managed line")?;
        }
        write!(writer, "{}", END_MARKER).context("Failed to write end marker")?;

        if !self.user_content.is_empty() {
            if !self.user_content.starts_with(['\n', '\r']) {
                writeln!(writer).context("Failed to write new line")?;
            }
            write!(writer, "{}", &self.user_content).context("Failed to write user content")?;
        }

        writer.flush().context("Failed to flush gitignore writer")?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use std::{
        io::{BufRead, BufReader, BufWriter, Write},
        ops::Add,
    };

    use crate::encryption_management::gitignore::{BEGIN_MARKER, END_MARKER, Gitignore};
    use lazin_test_utils::temp::TempFilepath;

    const LAZIN_V1_BEGIN: &str = "# >>> lazin v1 - PLEASE DO NOT MODIFY MANUALLY >>>\n";
    const LAZIN_V1_END: &str = "# <<< lazin - PLEASE DO NOT MODIFY MANUALLY <<<\n";

    fn expect_read<R: std::io::Read>(
        reader: &mut BufReader<R>,
        buffer: &mut String,
        expected: &str,
    ) {
        reader.read_line(buffer).expect("a line");
        assert_eq!(expected, buffer,);
        buffer.clear();
    }

    #[test]
    fn load_and_save() {
        let temp_file = TempFilepath::new();
        let test_managed_path = "test/test".to_string();
        let mut gitignore = Gitignore::load(temp_file.path()).expect("loaded gitignore");
        gitignore.managed.insert(test_managed_path.clone().into());
        gitignore.save().expect("a successfull save");

        let file = temp_file.file();
        let mut buffer = String::new();
        let mut reader = BufReader::new(file);

        expect_read(&mut reader, &mut buffer, LAZIN_V1_BEGIN);
        expect_read(&mut reader, &mut buffer, &test_managed_path.add("\n"));
        expect_read(
            &mut reader,
            &mut buffer,
            LAZIN_V1_END.strip_suffix('\n').unwrap(),
        );
    }

    #[test]
    fn load_and_save_with_user_data() {
        let temp_file = TempFilepath::new();
        let user_file_1 = "user/file".to_string();
        let user_file_2 = "another/file".to_string();
        {
            let file = temp_file.file();
            let mut writer = BufWriter::new(file);

            writeln!(writer, "{}", user_file_1).expect("a valid write");
            writeln!(writer, "{}", user_file_2).expect("a valid write");

            writer.flush().expect("a flush");
        }

        let test_managed_path = "test/test".to_string();
        let mut gitignore = Gitignore::load(temp_file.path()).expect("loaded gitignore");
        gitignore.managed.insert(test_managed_path.clone().into());
        gitignore.save().expect("a successfull save");

        let file = temp_file.file();
        let mut buffer = String::new();
        let mut reader = BufReader::new(file);

        expect_read(&mut reader, &mut buffer, LAZIN_V1_BEGIN);
        expect_read(&mut reader, &mut buffer, &test_managed_path.add("\n"));
        expect_read(&mut reader, &mut buffer, LAZIN_V1_END);
        expect_read(&mut reader, &mut buffer, &user_file_1.add("\n"));
        expect_read(&mut reader, &mut buffer, &user_file_2.add("\n"));
    }

    #[test]
    fn load_and_save_with_user_edit_generated_data() {
        let temp_file = TempFilepath::new();
        let user_file_1 = "user/file".to_string();
        let user_file_2 = "another/file".to_string();
        {
            let file = temp_file.file();
            let mut writer = BufWriter::new(file);

            writeln!(writer, "{}", user_file_1).expect("a valid write");

            writeln!(writer, "{}", BEGIN_MARKER).expect("a valid write");
            writeln!(writer, "manually edited value that should not exist",)
                .expect("a valid write");
            writeln!(writer, "{}", END_MARKER).expect("a valid write");

            writeln!(writer, "{}", user_file_2).expect("a valid write");

            writer.flush().expect("a flush");
        }

        let test_managed_path = "test/test".to_string();
        let mut gitignore = Gitignore::load(temp_file.path()).expect("loaded gitignore");
        gitignore.managed.insert(test_managed_path.clone().into());
        gitignore.save().expect("a successfull save");

        let file = temp_file.file();
        let mut buffer = String::new();
        let mut reader = BufReader::new(file);

        expect_read(&mut reader, &mut buffer, LAZIN_V1_BEGIN);
        expect_read(&mut reader, &mut buffer, &test_managed_path.add("\n"));
        expect_read(&mut reader, &mut buffer, LAZIN_V1_END);
        expect_read(&mut reader, &mut buffer, &user_file_1.add("\n"));
        expect_read(&mut reader, &mut buffer, &user_file_2.add("\n"));
    }

    #[test]
    fn load_add_entries_remove_entry() {
        let temp_file = TempFilepath::new();
        let user_file_1 = "user/file".to_string();
        let user_file_2 = "another/file".to_string();
        {
            let file = temp_file.file();
            let mut writer = BufWriter::new(file);

            writeln!(writer, "{}", user_file_1).expect("a valid write");
            writeln!(writer, "{}", user_file_2).expect("a valid write");

            writer.flush().expect("a flush");
        }

        let mut gitignore = Gitignore::load(temp_file.path()).expect("loaded gitignore");
        let test_managed_path = "test/test".to_string();
        let test_managed_path2 = "test/test2".to_string();
        gitignore.managed.insert(test_managed_path.clone().into());
        gitignore.managed.insert(test_managed_path2.clone().into());

        gitignore.save().expect("a successfull save");

        let file = temp_file.file();
        let mut buffer = String::new();
        let mut reader = BufReader::new(file);

        expect_read(&mut reader, &mut buffer, LAZIN_V1_BEGIN);
        expect_read(
            &mut reader,
            &mut buffer,
            &test_managed_path.clone().add("\n"),
        );
        expect_read(&mut reader, &mut buffer, &test_managed_path2.add("\n"));
        expect_read(&mut reader, &mut buffer, LAZIN_V1_END);
        expect_read(&mut reader, &mut buffer, &user_file_1.clone().add("\n"));
        expect_read(&mut reader, &mut buffer, &user_file_2.clone().add("\n"));

        let mut gitignore = Gitignore::load(temp_file.path()).expect("loaded gitignore");
        gitignore.managed.insert(test_managed_path.clone().into());
        gitignore.save().expect("a successfull save");

        let file = temp_file.file();
        let reader = BufReader::new(file);

        assert_eq!(5, reader.lines().count());
    }
}
