use failure::Error;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs::{rename, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::result;

#[macro_use]
extern crate failure;

pub type Result<T> = result::Result<T, Error>;

/// Every time this offset threshold is reached in the log file the KvStore will do a log compaction.
pub const KVS_ODFSET_THRESHOLD: u64 = 1_000_000;

/// Key-Value store structure.
pub struct KvStore {
    /// A HashMap from the std lib is used to store the key  and the log pointer of each element.
    m_hash: HashMap<String, u64>,

    /// File used to record the set and rm operations.
    file: File,

    path: PathBuf,
}

impl KvStore {
    // Compact the log file.
    fn compaction(&mut self) -> Result<()> {
        let temp_directory = env::temp_dir();
        let temp_file_name = temp_directory.join(".kvs.log");

        let temp_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&temp_file_name)?;
        let mut wr = BufWriter::new(&temp_file);

        for key in self.m_hash.keys() {
            let option_value = self.get(key.to_string())?;
            if let Some(value) = option_value {
                let command = Command::Set(key.to_string(), value);
                serde_json::to_writer(&mut wr, &command).unwrap();
                wr.write_all(b"\n")?;
            }
        }
        rename(temp_file_name, &self.path)?;

        Ok(())
    }

    // Initialize internal HashMap with the contents of the log file.
    fn init(&mut self) -> Result<()> {
        let mut reader = BufReader::new(&self.file);

        loop {
            let position = reader.seek(SeekFrom::Current(0))?;

            let mut line = String::new();
            let lize_size = reader.read_line(&mut line)?;

            if lize_size > 0 {
                let command: Command = serde_json::from_str(&line[..])?;
                match command {
                    Command::Rm(key) => self.m_hash.remove(&key),
                    Command::Set(key, _value) => self.m_hash.insert(key, position),
                };
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Assert if a key exists in the Key Value Storage.
    fn exists(&self, key: String) -> bool {
        match self.m_hash.get(&key) {
            Some(_s) => true,
            None => false,
        }
    }

    /// Get value of a given key in the KV store.
    ///
    ///```
    ///# use failure::Error;
    /// use kvs::KvStore;
    /// let store = KvStore::open(".")?;
    /// let missing = store.get("missing_key".to_owned())?;
    /// assert_eq!(missing, None);
    ///# Ok::<(), Error>(())
    ///```
    pub fn get(&self, key: String) -> Result<Option<String>> {
        let stored = self.m_hash.get(&key);
        match stored {
            Some(&position) => {
                let mut reader = BufReader::new(&self.file);
                let mut line = String::new();
                reader.seek(SeekFrom::Start(position))?;
                reader.read_line(&mut line)?;
                let command: Command = serde_json::from_str(&line[..])?;
                let value = match command {
                    Command::Set(_key, value) => value,
                    _ => panic!("Invalid state"),
                };

                Ok(Some(value))
            }
            _ => Ok(None),
        }
    }

    fn log(&mut self, command: &Command) -> Result<u64> {
        let mut wr = BufWriter::new(&self.file);
        let position = wr.seek(SeekFrom::End(0))?;
        serde_json::to_writer(&mut wr, &command).unwrap();
        wr.write_all(b"\n")?;
        Ok(position)
    }

    ///
    /// Create a KvStore in a given path.
    /// ```
    ///# use failure::Error;
    /// use kvs::KvStore;
    /// let mut store = KvStore::open(".")?;
    ///# Ok::<(), Error>(())
    /// ```
    pub fn open(path: impl Into<PathBuf>) -> Result<KvStore> {
        let mut path: PathBuf = path.into();

        path.push("kvs.log");

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&path)?;

        let m_hash = HashMap::new();

        let mut storage = KvStore { m_hash, file, path };

        storage.init()?;

        Ok(storage)
    }

    ///
    /// Set value of a key in in the KV store
    /// ```
    ///# use failure::Error;
    /// use kvs::KvStore;
    /// let mut store = KvStore::open(".")?;
    /// store.set("key1".to_owned(), "value1".to_owned())?;
    /// let value = store.get("key1".to_owned())?.unwrap();
    /// assert_eq!(value, "value1");
    ///# Ok::<(), Error>(())
    /// ```
    pub fn set(&mut self, key: String, value: String) -> Result<()> {
        let command = Command::Set(key.clone(), value.clone());
        let position = self.log(&command)?;
        self.m_hash.insert(key, position);

        if position > KVS_ODFSET_THRESHOLD {
            self.compaction()?;
        }
        Ok(())
    }

    /// Remove key-value from the KV store
    /// ```
    ///# use failure::Error;
    /// use kvs::KvStore;
    /// let mut store = KvStore::open(".")?;
    /// store.set("key1".to_owned(), "value1".to_owned())?;
    /// let value = store.get("key1".to_owned())?.unwrap();
    /// assert_eq!(value, "value1");
    /// store.remove("key1".to_owned());
    /// let missing = store.get("missing_key".to_owned())?;
    /// assert_eq!(missing, None);
    ///# Ok::<(), Error>(())
    /// ```
    pub fn remove(&mut self, key: String) -> Result<()> {
        let command = Command::Rm(key.clone());
        if self.exists(key.clone()) {
            self.log(&command)?;
            self.m_hash.remove(&key);
            Ok(())
        } else {
            Err(format_err!("Key not found"))
        }
    }
}

#[derive(Serialize, Deserialize, Debug)]
enum Command {
    Rm(String),
    Set(String, String),
}
