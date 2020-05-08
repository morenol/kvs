use crate::error::{Error, ErrorKind, Result};

use super::KvsEngine;

use crate::command::Command;
use std::collections::HashMap;
use std::env;
use std::fs::{rename, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU16, Ordering};
use std::sync::{Arc, Mutex, RwLock};

/// Every time this offset threshold is reached in the log file the KvStore will do a log compaction.
pub const KVS_UNCOMPACTED_THRESHOLD: u16 = 4_000;

/// Key-Value store structure.
#[derive(Clone)]
pub struct KvStore {
    /// A HashMap from the std lib is used to store the key  and the log pointer of each element.
    index: Arc<RwLock<HashMap<String, u64>>>,

    reader: Arc<RwLock<BufReader<File>>>,
    writer: Arc<Mutex<BufWriter<File>>>,

    // Number of write operations since last compactation
    uncompacted: Arc<AtomicU16>,

    path: PathBuf,
}

impl KvStore {
    // Compact the log file.
    fn compaction(&self) -> Result<()> {
        let temp_directory = env::temp_dir();
        let temp_file_name = temp_directory.join(".kvs.log");
        let mut new_index = HashMap::new();
        let temp_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&temp_file_name)
            .map_err(|_err| Error::from(ErrorKind::FileError))?;
        let mut wr = BufWriter::new(temp_file);

        let mut lock = self.writer.lock().unwrap();

        for key in self.index.read().unwrap().keys() {
            let position = wr.seek(SeekFrom::Current(0)).unwrap();
            let option_value = self.get(key.to_owned())?;
            if let Some(value) = option_value {
                let command = Command::Set(key.to_owned(), value);
                serde_json::to_writer(&mut wr, &command).unwrap();
                wr.write_all(b"\n")
                    .map_err(|_err| Error::from(ErrorKind::FileError))?;
            }
            new_index.insert(key.to_owned(), position);
        }
        wr.flush().unwrap();

        {
            let mut r = self.reader.write().unwrap();
            rename(temp_file_name, &self.path).map_err(|_err| Error::from(ErrorKind::FileError))?;
            let mut index = self.index.write().unwrap();
            self.uncompacted.store(0, Ordering::SeqCst);
            *index = new_index;
            let file = OpenOptions::new()
                .read(true)
                .write(true)
                .open(&self.path)
                .map_err(|_err| Error::from(ErrorKind::FileError))?;
            let reader = BufReader::new(file);
            *r = reader;
        }

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&self.path)
            .map_err(|_err| Error::from(ErrorKind::FileError))?;
        let mut writer = BufWriter::new(file);
        writer.seek(SeekFrom::End(0)).unwrap();
        *lock = writer;

        Ok(())
    }

    // Initialize internal HashMap with the contents of the log file.
    fn init(&mut self) -> Result<()> {
        loop {
            let position = self
                .reader
                .write()
                .unwrap()
                .seek(SeekFrom::Current(0))
                .map_err(|_err| Error::from(ErrorKind::FileError))?;

            let mut line = String::new();
            let lize_size = self
                .reader
                .write()
                .unwrap()
                .read_line(&mut line)
                .map_err(|_err| Error::from(ErrorKind::FileError))?;

            if lize_size > 0 {
                let command: Command = serde_json::from_str(&line[..])
                    .map_err(|_err| Error::from(ErrorKind::ParsingError))?;
                match command {
                    Command::Rm(key) => self.index.write().unwrap().remove(&key),
                    Command::Set(key, _value) => self.index.write().unwrap().insert(key, position),
                    _ => panic!("error"),
                };
            } else {
                break;
            }
        }
        Ok(())
    }

    /// Assert if a key exists in the Key Value Storage.
    fn exists(&self, key: String) -> bool {
        match self.index.read().unwrap().get(&key) {
            Some(_s) => true,
            None => false,
        }
    }

    fn log(&self, command: &Command) -> Result<u64> {
        let mut wr = self.writer.lock().unwrap();
        let position = wr
            .seek(SeekFrom::Current(0))
            .map_err(|_err| Error::from(ErrorKind::FileError))?;
        serde_json::to_writer(&mut *wr, &command).unwrap();
        wr.write_all(b"\n")
            .map_err(|_err| Error::from(ErrorKind::FileError))?;
        wr.flush().unwrap();
        Ok(position)
    }

    ///
    /// Create a KvStore in a given path.
    /// ```
    /// use kvs::{KvStore, KvsEngine};
    /// use kvs::error::Error;
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
            .open(&path)
            .map_err(|_err| Error::from(ErrorKind::FileError))?;

        let reader = Arc::new(RwLock::new(BufReader::new(file)));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .open(&path)
            .map_err(|_err| Error::from(ErrorKind::FileError))?;
        let mut writer = BufWriter::new(file);
        writer.seek(SeekFrom::End(0)).unwrap();
        let writer = Arc::new(Mutex::new(writer));
        let index = Arc::new(RwLock::new(HashMap::new()));
        let uncompacted = Arc::new(AtomicU16::new(0));

        let mut storage = KvStore {
            index,
            reader,
            writer,
            path,
            uncompacted,
        };

        storage.init()?;

        Ok(storage)
    }
}

impl KvsEngine for KvStore {
    /// Get value of a given key in the KV store.
    ///
    ///```
    /// use kvs::{KvStore, KvsEngine};
    /// use kvs::error::Error;
    /// let store = KvStore::open(".")?;
    /// let missing = store.get("missing_key".to_owned())?;
    /// assert_eq!(missing, None);
    ///# Ok::<(), Error>(())
    ///```
    fn get(&self, key: String) -> Result<Option<String>> {
        let mut reader = self.reader.write().unwrap();
        match self.index.read().unwrap().get(&key) {
            Some(&position) => {
                let mut line = String::new();
                reader
                    .seek(SeekFrom::Start(position))
                    .map_err(|_err| Error::from(ErrorKind::FileError))?;
                reader
                    .read_line(&mut line)
                    .map_err(|_err| Error::from(ErrorKind::FileError))?;
                let command: Command = serde_json::from_str(&line[..])
                    .map_err(|_err| Error::from(ErrorKind::ParsingError))?;
                let value = match command {
                    Command::Set(_key, value) => value,
                    _ => return Err(Error::from(ErrorKind::InvalidData)),
                };

                Ok(Some(value))
            }
            _ => Ok(None),
        }
    }

    ///
    /// Set value of a key in in the KV store
    /// ```
    /// use kvs::{KvStore, KvsEngine};
    /// use kvs::error::Error;
    /// let mut store = KvStore::open(".")?;
    /// store.set("key1".to_owned(), "value1".to_owned())?;
    /// let value = store.get("key1".to_owned())?.unwrap();
    /// assert_eq!(value, "value1");
    ///# Ok::<(), Error>(())
    /// ```
    fn set(&self, key: String, value: String) -> Result<()> {
        let command = Command::Set(key.clone(), value);
        let position = self.log(&command)?;
        {
            self.index.write().unwrap().insert(key, position);
        }
        if self.uncompacted.fetch_add(1, Ordering::Relaxed) > KVS_UNCOMPACTED_THRESHOLD {
            self.compaction()?;
        }
        Ok(())
    }

    /// Remove key-value from the KV store
    /// ```
    /// use kvs::{KvStore, KvsEngine};
    /// use kvs::error::Error;
    /// let mut store = KvStore::open(".")?;
    /// store.set("key1".to_owned(), "value1".to_owned())?;
    /// let value = store.get("key1".to_owned())?.unwrap();
    /// assert_eq!(value, "value1");
    /// store.remove("key1".to_owned());
    /// let missing = store.get("missing_key".to_owned())?;
    /// assert_eq!(missing, None);
    ///# Ok::<(), Error>(())
    /// ```
    fn remove(&self, key: String) -> Result<()> {
        let command = Command::Rm(key.clone());
        if self.exists(key.clone()) {
            self.log(&command)?;
            {
                self.index.write().unwrap().remove(&key);
            }
            self.uncompacted.fetch_add(1, Ordering::Relaxed);
            if self.uncompacted.fetch_add(1, Ordering::Relaxed) > KVS_UNCOMPACTED_THRESHOLD {
                self.compaction()?;
            }
            Ok(())
        } else {
            Err(Error::from(ErrorKind::KeyNotFound))
        }
    }
}
