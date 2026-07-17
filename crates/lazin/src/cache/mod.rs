use std::{
    collections::{BTreeMap, btree_map},
    fs::{self, File},
    io::{BufWriter, ErrorKind, Read, Write},
    path::{Path, PathBuf},
};

use lazin_error::{Context, LazinResult};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

const CACHE_FILE: &str = "lazin.cache";

#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct FileHash(String);

impl FileHash {
    pub fn hash<P: AsRef<Path>>(path: P) -> LazinResult<Self> {
        let mut file = File::open(path).context("Failed to open file for hashing")?;
        let mut hasher = Sha256::new();
        let mut buffer = [0u8; 8192];

        loop {
            let n = file
                .read(&mut buffer)
                .context("Failed to read hashing file contents")?;
            if n == 0 {
                break;
            }
            hasher.update(&buffer[..n]);
        }

        let digest = hasher.finalize();
        Ok(Self(Self::encode(digest)))
    }

    fn encode(digest: sha2::digest::Output<sha2::Sha256>) -> String {
        digest.iter().map(|b| format!("{:02x}", b)).collect()
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct CacheEntries {
    encryption: BTreeMap<PathBuf, FileHash>,
}

#[derive(Debug)]
pub struct Cache {
    file_path: PathBuf,
    entries: CacheEntries,
}

pub enum Entry {
    Encryption(PathBuf, FileHash),
}

pub enum CompareEntry {
    Encryption(PathBuf, FileHash),
}

pub enum EntryComparison {
    Equal,
    NotEqual,
    NotFound,
}

impl Cache {
    pub fn try_new<P: AsRef<Path>>(cache_dir: P) -> LazinResult<Self> {
        let cache_path = cache_dir.as_ref().join(CACHE_FILE);

        match cache_path.try_exists() {
            Ok(true) => {}
            Ok(false) => init_cache(&cache_path)?,
            Err(e) if e.kind() == ErrorKind::NotFound => init_cache(&cache_path)?,
            Err(e) => return Err(e).context("Failed to check if cache file exists"),
        };

        let mut file_data = String::new();
        fs::File::open(&cache_path)
            .context("Failed to open cache file")?
            .read_to_string(&mut file_data)
            .context("Failed to read cache file")?;
        let entries = toml::from_str(&file_data)?;

        Ok(Self {
            file_path: cache_path,
            entries,
        })
    }

    pub fn save(&self) -> LazinResult {
        let serialized_entries = toml::to_string(&self.entries)?;

        let file = fs::OpenOptions::new().write(true).open(&self.file_path)?;
        let mut writer = BufWriter::new(file);

        writeln!(
            writer,
            "# This file is generated and should not be modified by hand."
        )?;
        writeln!(
            writer,
            "# You can safely commit this file into source control."
        )?;
        writeln!(writer)?;

        write!(writer, "{}", serialized_entries)?;

        writer.flush()?;
        Ok(())
    }

    pub fn prune_entries(&mut self, entries_to_keep: &[PathBuf]) {
        self.entries
            .encryption
            .retain(|k, _| entries_to_keep.contains(k));
    }

    pub fn add_entry(&mut self, entry: Entry) -> LazinResult<Option<Entry>> {
        let old_entry = match entry {
            Entry::Encryption(path_buf, hash) => match self.entries.encryption.entry(path_buf) {
                btree_map::Entry::Vacant(entry) => {
                    entry.insert(hash);
                    None
                }
                btree_map::Entry::Occupied(mut entry) => {
                    let old_hash = entry.insert(hash);
                    let path_buf = entry.key().clone();

                    Some(Entry::Encryption(path_buf, old_hash))
                }
            },
        };

        Ok(old_entry)
    }

    pub fn compare_entry(&self, entry: CompareEntry) -> EntryComparison {
        match entry {
            CompareEntry::Encryption(path_buf, file_hash) => self
                .entries
                .encryption
                .get(&path_buf)
                .map_or(EntryComparison::NotFound, |entry_hash| {
                    if entry_hash == &file_hash {
                        EntryComparison::Equal
                    } else {
                        EntryComparison::NotEqual
                    }
                }),
        }
    }
}

fn init_cache(cache_file: &Path) -> LazinResult<()> {
    fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(cache_file)?;

    Ok(())
}
