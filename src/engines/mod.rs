use crate::command::Command;
use crate::error::{Error, ErrorKind, Result};
use sled::Db;
use std::collections::HashMap;
use std::env;
use std::fs::{rename, File, OpenOptions};
use std::io::{BufRead, BufReader, BufWriter, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub trait KvsEngine: Clone + Send + 'static {
    fn set(&self, key: String, value: String) -> Result<()>;

    fn get(&self, key: String) -> Result<Option<String>>;

    fn remove(&self, key: String) -> Result<()>;
}

/// Every time this offset threshold is reached in the log file the KvStore will do a log compaction.
pub const KVS_ODFSET_THRESHOLD: u64 = 100;

/// Key-Value store structure.
#[derive(Clone)]
pub struct KvStore {
    /// A HashMap from the std lib is used to store the key  and the log pointer of each element.
    m_hash: Arc<Mutex<HashMap<String, u64>>>,

    /// File used to record the set and rm operations.
    file: Arc<Mutex<File>>,

    path: PathBuf,
}

impl KvStore {
    // Compact the log file.
    fn compaction(&self) -> Result<()> {
        let temp_directory = env::temp_dir();
        let temp_file_name = temp_directory.join(".kvs.log");

        let temp_file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&temp_file_name)
            .map_err(|_err| Error::from(ErrorKind::FileError))?;
        let mut wr = BufWriter::new(&temp_file);
        let _file = self.file.lock().unwrap();
        for key in self.m_hash.lock().unwrap().keys() {
            let option_value = self.get(key.to_owned())?;
            if let Some(value) = option_value {
                let command = Command::Set(key.to_owned(), value);
                serde_json::to_writer(&mut wr, &command).unwrap();
                wr.write_all(b"\n")
                    .map_err(|_err| Error::from(ErrorKind::FileError))?;
            }
        }
        rename(temp_file_name, &self.path).map_err(|_err| Error::from(ErrorKind::FileError))?;

        Ok(())
    }

    // Initialize internal HashMap with the contents of the log file.
    fn init(&mut self) -> Result<()> {
        let file = &*self.file.lock().unwrap();
        let mut reader = BufReader::new(file);

        loop {
            let position = reader
                .seek(SeekFrom::Current(0))
                .map_err(|_err| Error::from(ErrorKind::FileError))?;

            let mut line = String::new();
            let lize_size = reader
                .read_line(&mut line)
                .map_err(|_err| Error::from(ErrorKind::FileError))?;

            if lize_size > 0 {
                let command: Command = serde_json::from_str(&line[..])
                    .map_err(|_err| Error::from(ErrorKind::ParsingError))?;
                match command {
                    Command::Rm(key) => self.m_hash.lock().unwrap().remove(&key),
                    Command::Set(key, _value) => self.m_hash.lock().unwrap().insert(key, position),
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
        match self.m_hash.lock().unwrap().get(&key) {
            Some(_s) => true,
            None => false,
        }
    }

    fn log(&self, command: &Command) -> Result<u64> {
        let file = &*self.file.lock().unwrap();

        let mut wr = BufWriter::new(file);
        let position = wr
            .seek(SeekFrom::End(0))
            .map_err(|_err| Error::from(ErrorKind::FileError))?;
        serde_json::to_writer(&mut wr, &command).unwrap();
        wr.write_all(b"\n")
            .map_err(|_err| Error::from(ErrorKind::FileError))?;
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

        let file = Arc::new(Mutex::new(file));
        let m_hash = Arc::new(Mutex::new(HashMap::new()));

        let mut storage = KvStore { m_hash, file, path };

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
        //        let stored = self.m_hash.lock().unwrap().get(&key);
        match self.m_hash.lock().unwrap().get(&key) {
            Some(&position) => {
                let file = &*self.file.lock().unwrap();
                let mut reader = BufReader::new(file);
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
            self.m_hash.lock().unwrap().insert(key, position);
        }
        //if position > KVS_ODFSET_THRESHOLD {
        //    self.compaction()?;
        //}
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
                self.m_hash.lock().unwrap().remove(&key);
            }
            Ok(())
        } else {
            Err(Error::from(ErrorKind::KeyNotFound))
        }
    }
}

#[derive(Clone)]
pub struct SledStore {
    store: Db,
}
impl SledStore {
    pub fn open(path: impl Into<PathBuf>) -> Result<Self> {
        let path: PathBuf = path.into();
        let st = sled::Config::new().path(path).flush_every_ms(None).open();
        match st {
            Ok(store) => Ok(SledStore { store }),
            Err(_err) => Err(Error::from(ErrorKind::SledError)),
        }
    }
}
impl KvsEngine for SledStore {
    fn get(&self, key: String) -> Result<Option<String>> {
        let result = self.store.get(key);
        match result {
            Ok(Some(s)) => {
                let value = String::from_utf8(s.to_vec());
                match value {
                    Ok(vl) => Ok(Some(vl)),
                    _ => Err(Error::from(ErrorKind::SledError)),
                }
            }
            Ok(None) => Ok(None),
            Err(_) => Err(Error::from(ErrorKind::SledError)),
        }
    }

    fn set(&self, key: String, value: String) -> Result<()> {
        let result = self.store.insert(key, value.as_bytes());
        match result {
            Ok(_something) => {
                let res = self.store.flush();
                match res {
                    Ok(_something) => Ok(()),
                    Err(_err) => Err(Error::from(ErrorKind::SledError)),
                }
            }
            Err(_) => Err(Error::from(ErrorKind::SledError)),
        }
    }

    fn remove(&self, key: String) -> Result<()> {
        let result = self.store.remove(key);
        match result {
            Ok(Some(_thing)) => {
                let res = self.store.flush();
                match res {
                    Ok(_something) => Ok(()),
                    Err(_err) => Err(Error::from(ErrorKind::SledError)),
                }
            }
            Ok(None) => Err(Error::from(ErrorKind::KeyNotFound)),
            Err(_err) => Err(Error::from(ErrorKind::UnknownError)),
        }
    }
}

#[derive(Clone)]
pub enum Engine {
    KvsEngine(KvStore),
    SledKvsEngine(SledStore),
}

impl KvsEngine for Engine {
    fn get(&self, key: String) -> Result<Option<String>> {
        match self {
            Engine::KvsEngine(engine) => engine.get(key),
            Engine::SledKvsEngine(engine) => engine.get(key),
        }
    }

    fn set(&self, key: String, value: String) -> Result<()> {
        match self {
            Engine::KvsEngine(engine) => engine.set(key, value),
            Engine::SledKvsEngine(engine) => engine.set(key, value),
        }
    }

    fn remove(&self, key: String) -> Result<()> {
        match self {
            Engine::KvsEngine(engine) => engine.remove(key),
            Engine::SledKvsEngine(engine) => engine.remove(key),
        }
    }
}

impl Engine {
    pub fn exec_command(&self, command: Command) -> Result<Option<String>> {
        match command {
            Command::Rm(key) => {
                self.remove(key)?;
                Ok(None)
            }
            Command::Set(key, value) => {
                self.set(key, value)?;
                Ok(None)
            }
            Command::Get(key) => self.get(key),
        }
    }
}
